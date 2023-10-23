
This tiny crate adds another way to ordering systems for Bevy.
Systems can be ordered by dependencies to datas.

For example, consider `system1` writing `ComponentA` and `system2` reading `ComponentA`.

In standard form, we can add and order them with the code below.
```
app.add_systems(Update, system1);
app.add_systems(Update, system2.after(system1));
```

With this crate, you can add and order them as below, labeling systems with the types which the system read or write.
```
add_data_flow(app, Update, system1, (), DataLabel::<ComponentA>::default());
add_data_flow(app, Update, system2, DataLabel::<ComponentA>::default(), ());
```

In this form, we don't need to list all systems which may write to `ComponentA` when adding `system2`.
We can just think about the indivisual system at the time.


This crate may be outdated after Bevy itself solve this issue.
* https://github.com/bevyengine/bevy/issues/7857
* https://github.com/bevyengine/bevy/pull/8595


## Limitations
* Can't handle system which read and write on same type. It is considered as cyclic dependency.
* You have to manually list the `DataLabel<T>` which the system is related.
    Automatic listing from arguments of the system looks like beyond my capacity.


## License
This crate is dual-licensed under MIT and Apache 2.0 at your option.

Feel free to merge this crate to yours or Bevy itself.
I'd hope there are more smarter implements.


## Contributing
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
