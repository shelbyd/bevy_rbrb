use ::serde::{de::DeserializeSeed, *};
use bevy_ecs::{component::Component, prelude::*};
use bevy_reflect::{
    serde::{ReflectDeserializer, ReflectSerializer},
    *,
};
use std::collections::BTreeMap;

use crate::RollbackId;

#[derive(Default)]
pub struct Snapshotter {
    component_registry: TypeRegistry,
}

impl Snapshotter {
    pub fn register_component<T: RegisterComponent>(&mut self) {
        self.component_registry.register::<T>();
        let registration = self
            .component_registry
            .get_mut(std::any::TypeId::of::<T>())
            .unwrap();
        registration.insert(<ReflectComponent as FromType<T>>::from_type());
    }

    pub fn save_to(&mut self, vec: &mut Vec<u8>, world: &mut World) {
        let mut snapshot = Snapshot::default();
        snapshot.fill_entities(world, &self.component_registry);
        bincode::serialize_into(vec, &snapshot).unwrap();
    }

    pub fn load_from(&mut self, slice: &[u8], world: &mut World) {
        let mut snapshot: Snapshot = bincode::deserialize(slice).unwrap();
        snapshot.apply_to(world, &self.component_registry);
    }
}

pub trait RegisterComponent: Component + GetTypeRegistration + Reflect + Default {}

impl<T> RegisterComponent for T where T: Component + GetTypeRegistration + Reflect + Default {}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct ComponentName(pub String);

#[derive(Default, Serialize, Deserialize)]
struct Snapshot {
    entities: BTreeMap<RollbackId, BTreeMap<ComponentName, Vec<u8>>>,
}

impl Snapshot {
    fn fill_entities(&mut self, world: &World, registry: &TypeRegistry) {
        self.entities.clear();
        for arch in world.archetypes().iter() {
            let entity_map: BTreeMap<_, _> = arch
                .entities()
                .iter()
                .filter_map(|e| Some((*e, world.get::<RollbackId>(*e)?)))
                .collect();

            for component_id in arch.components() {
                let component_info = match world.components().get_info(component_id) {
                    Some(i) => i,
                    None => continue,
                };
                let component_name = component_info.name();
                let reflect = match registry
                    .get(component_info.type_id().unwrap())
                    .and_then(|reg| reg.data::<ReflectComponent>())
                {
                    Some(r) => r,
                    None => continue,
                };
                for (entity, &rollback) in &entity_map {
                    let component = match reflect.reflect_component(world, *entity) {
                        Some(c) => c,
                        None => continue,
                    };
                    let serializer = ReflectSerializer::new(component, registry);
                    let serialized = bson::to_vec(&serializer).unwrap();
                    self.entities
                        .entry(rollback.clone())
                        .or_default()
                        .insert(ComponentName(component_name.to_owned()), serialized);
                }
            }
        }
    }

    fn apply_to(&mut self, world: &mut World, registry: &TypeRegistry) {
        let mut to_update = world.query::<(Entity, &RollbackId)>();
        let to_update = to_update
            .iter(world)
            .map(|(entity, rollback)| (entity, rollback.clone()))
            .collect::<Vec<_>>();

        for (entity, rollback) in to_update {
            let components = self.entities.remove(&rollback).expect(&format!(
                "found entity with missing rollback data {}",
                rollback.0
            ));
            self.apply_components_to(components, entity, registry, world);
        }
        for (rollback, components) in std::mem::take(&mut self.entities) {
            let entity = world.spawn().insert(rollback).id();
            self.apply_components_to(components, entity, registry, world);
        }
    }

    fn apply_components_to(
        &mut self,
        mut components: BTreeMap<ComponentName, Vec<u8>>,
        entity: Entity,
        registry: &TypeRegistry,
        world: &mut World,
    ) {
        for registration in registry.iter() {
            let type_id = registration.type_id();
            let reflect = registration.data::<ReflectComponent>().unwrap();

            let component = components
                .remove(&ComponentName(registration.name().to_string()))
                .map(|data| {
                    let bson = bson::from_slice(&data).unwrap();
                    let de = bson::Deserializer::new(bson);
                    let world_registry = world.get_resource::<TypeRegistryArc>().unwrap().read();
                    ReflectDeserializer::new(&world_registry).deserialize(de).unwrap()
                });
            match (world.entity(entity).contains_type_id(type_id), component) {
                (true, Some(c)) => {
                    reflect.apply_component(world, entity, &*c);
                }
                (false, Some(c)) => {
                    reflect.add_component(world, entity, &*c);
                }
                (true, None) => {
                    reflect.remove_component(world, entity);
                }
                (false, None) => {}
            }
        }
    }
}
