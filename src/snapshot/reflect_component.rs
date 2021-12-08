//! Types that enable reflection support.

pub use bevy_ecs::reflect::ReflectMut;
use bevy_ecs::{
    component::Component,
    entity::Entity,
    world::{FromWorld, World},
};
use bevy_reflect::{FromType, Reflect};

#[derive(Clone)]
pub struct ReflectComponent {
    add_component: fn(&mut World, Entity, &dyn Reflect),
    apply_component: fn(&mut World, Entity, &dyn Reflect),
    remove_component: fn(&mut World, Entity),
    reflect_component: fn(&World, Entity) -> Option<&dyn Reflect>,
}

impl ReflectComponent {
    pub fn add_component(&self, world: &mut World, entity: Entity, component: &dyn Reflect) {
        (self.add_component)(world, entity, component);
    }

    pub fn apply_component(&self, world: &mut World, entity: Entity, component: &dyn Reflect) {
        (self.apply_component)(world, entity, component);
    }

    pub fn remove_component(&self, world: &mut World, entity: Entity) {
        (self.remove_component)(world, entity);
    }

    pub fn reflect_component<'a>(
        &self,
        world: &'a World,
        entity: Entity,
    ) -> Option<&'a dyn Reflect> {
        (self.reflect_component)(world, entity)
    }
}

impl<C: Component + Reflect + FromWorld> FromType<C> for ReflectComponent {
    fn from_type() -> Self {
        ReflectComponent {
            add_component: |world, entity, reflected_component| {
                let mut component = C::from_world(world);
                component.apply(reflected_component);
                world.entity_mut(entity).insert(component);
            },
            apply_component: |world, entity, reflected_component| {
                let mut component = world.get_mut::<C>(entity).unwrap();
                component.apply(reflected_component);
            },
            remove_component: |world, entity| {
                world.entity_mut(entity).remove::<C>();
            },
            reflect_component: |world, entity| {
                world
                    .get_entity(entity)?
                    .get::<C>()
                    .map(|c| c as &dyn Reflect)
            },
        }
    }
}
