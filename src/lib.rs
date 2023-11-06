use std::{marker::PhantomData, hash::Hasher};

use bevy::{prelude::*, utils::label::{DynEq, DynHash}, ecs::schedule::{ScheduleLabel, SystemConfigs}};
use tiny_utils_proc_macros::all_tuples_with_index;


/// Util function to order systems by dependency of datas.
///
/// # Arguments
/// * `system`  - The system which you want to describe.
/// * `read_sets`     - Collection of `DataLabel<T>` where the system read `T`.
/// * `write_sets`    - Collection of `DataLabel<T>` where the system may write to `T`.
pub fn add_data_flow<Marker>(
    app: &mut App,
    schedule: impl ScheduleLabel,
    system: impl IntoSystemConfigs<Marker>,
    read_sets: impl DataLabels,
    write_sets: impl DataLabels,
) {
    let config = system;
    let config = read_sets.mark_read(config);
    let config = write_sets.mark_write(config);
    app.add_systems(schedule, config);
}


/// `SystemSet` to represent relation to the data.
/// Generics type parameter `T` is the container of the data, usually implements `Component` or `Resource`.
//
// derive macro doesn't work well now.
// maybe https://github.com/rust-lang/rust/issues/26925
// #[derive(SystemSet, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DataLabel<T: 'static + Send + Sync>(PhantomData<T>);

impl<T: 'static + Send + Sync> Default for DataLabel<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<T: 'static + Send + Sync> Clone for DataLabel<T> {
    fn clone(&self) -> Self {
        Self::default()
    }
}
impl<T: 'static + Send + Sync> Copy for DataLabel<T> {}
impl<T: 'static + Send + Sync> PartialEq for DataLabel<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl<T: 'static + Send + Sync> Eq for DataLabel<T> {}
impl<T: 'static + Send + Sync> std::fmt::Debug for DataLabel<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f
            .debug_struct(std::any::type_name::<Self>())
            .finish()
    }
}
impl<T: 'static + Send + Sync> std::hash::Hash for DataLabel<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::any::type_name::<Self>().hash(state)   // probably not needed
    }
}
impl<T: 'static + Send + Sync> SystemSet for DataLabel<T> {
    fn as_dyn_eq(&self) -> &(dyn DynEq + 'static) {
        DynHash::as_dyn_eq(self)
    }
    fn dyn_hash(&self, state: &mut dyn Hasher) {
        DynHash::dyn_hash(self, state);
    }
    fn dyn_clone(&self) -> Box<dyn SystemSet> {
        Box::new(self.clone())
    }
}


/// Collection of `DataLabel`.
/// Implemented to `DataLabel<T>`, `()`, `(T0: DataLabels, )`, `(T0: DataLabels, T1: DataLabels)`, ... 
pub trait DataLabels {
    fn mark_read<Marker>(&self, config: impl IntoSystemConfigs<Marker>) -> SystemConfigs;
    fn mark_write<Marker>(&self, config: impl IntoSystemConfigs<Marker>) -> SystemConfigs;
}

impl<T> DataLabels for DataLabel<T>
where
    T: 'static + Send + Sync,
{
    fn mark_read<Marker>(&self, config: impl IntoSystemConfigs<Marker>) -> SystemConfigs {
        config.after(*self)
    }
    fn mark_write<Marker>(&self, config: impl IntoSystemConfigs<Marker>) -> SystemConfigs {
        config.in_set(*self)
    }
}
impl DataLabels for () {
    fn mark_read<Marker>(&self, config: impl IntoSystemConfigs<Marker>) -> SystemConfigs {
        config.run_if(|| true)  // into SystemConfigs
    }
    fn mark_write<Marker>(&self, config: impl IntoSystemConfigs<Marker>) -> SystemConfigs {
        config.run_if(|| true)  // into SystemConfigs
    }
}

macro_rules! impl_data_labels_tuple {
    ($(($idx: tt, $label: ident)),*) => {
        impl<$($label: DataLabels),*> DataLabels for ($($label,)*) {
            fn mark_read<Marker>(&self, config: impl IntoSystemConfigs<Marker>) -> SystemConfigs {
                $(let config = self.$idx.mark_read(config);)*
                config
            }
            fn mark_write<Marker>(&self, config: impl IntoSystemConfigs<Marker>) -> SystemConfigs {
                $(let config = self.$idx.mark_write(config);)*
                config
            }
        }
    };
}
all_tuples_with_index!(impl_data_labels_tuple, 1, 15, T);





#[cfg(test)]
mod tests {
    use super::*;
    use bevy::core::FrameCount;

    #[derive(Component, Debug)]
    struct ComponentA(pub u32);

    #[derive(Component, Debug)]
    struct ComponentB(pub u32);

    #[derive(Resource, Debug, Default)]
    struct ResourceC(pub Vec<u32>);

    fn increment_a (mut query: Query<&mut ComponentA>) {
        for mut comp_a in query.iter_mut() {
            comp_a.0 += 1;
        }
    }
    fn double_a_to_b (mut query: Query<(&ComponentA, &mut ComponentB)>) {
        for (comp_a, mut comp_b) in query.iter_mut() {
            comp_b.0 = 2 * comp_a.0;
        }
    }
    fn push_b_to_c (query: Query<&ComponentB>, mut res_c: ResMut<ResourceC>) {
        for comp_b in query.iter() {
            res_c.0.push(comp_b.0);
        }
    }
    fn aseert_system (query: Query<(&ComponentA, &ComponentB)>, res_c: Res<ResourceC>, frame_count: Res<FrameCount>) {
        for (comp_a, comp_b) in query.iter() {
            assert!(
                comp_a.0 == frame_count.0 + 1,
                "Value of ComponentA ({}) should equal to count ({}) + 1.", comp_a.0, frame_count.0
            );
            assert!(
                comp_b.0 == 2 * (frame_count.0 + 1),
                "Value of ComponentB ({}) should equal to 2 * (count ({}) + 1).", comp_b.0, frame_count.0
            );
        }
        let expedted_log: Vec<u32> = (1..(frame_count.0 + 2)).map(|v| 2*v).collect();
        assert!(
            res_c.0 == expedted_log,
            "Value of ResourceC ({:?}) should equal to {:?}", res_c.0, expedted_log
        );
    }

    #[test]
    fn test_order() {
        let mut app = App::new();
        app.add_plugins(FrameCountPlugin);
        app.world.init_resource::<ResourceC>();
        app.world.spawn((ComponentA(0), ComponentB(0)));
        add_data_flow(
            &mut app, Update, aseert_system,
            (DataLabel::<ComponentA>::default(), DataLabel::<ComponentB>::default(), DataLabel::<ResourceC>::default()),
            ()
        );
        add_data_flow(&mut app, Update, push_b_to_c, DataLabel::<ComponentB>::default(), DataLabel::<ResourceC>::default());
        add_data_flow(&mut app, Update, increment_a, (), DataLabel::<ComponentA>::default());
        add_data_flow(&mut app, Update, double_a_to_b, DataLabel::<ComponentA>::default(), DataLabel::<ComponentB>::default());
        app.update();
        app.update();
        app.update();
    }
}

