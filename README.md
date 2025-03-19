# Playground for experiments with Ixa internals

This repo is a playground for experiments with [Ixa](https://github.com/CDCgov/ixa) internals. For example: 
-  `People` are generalized to `Entities`
- `EntityData` (`PeopleData`) only uses one `RefCell` in one of its fields
- the pattern of `(PropertyTagType, PropertyValueType)` is replaced with just `PropertyValueType`
- several instances of panics are replaced with lazy instantiation
- queries are very different and are responsible for executing themselves
- `Property` dependencies are computed differently, with derived properties responsible for computing their own dependencies.
- All nonpublic API is moved to private implementations (sort of, it's a work in progress)

...and so on. These are experiments which may or may not trickle down to the actual Ixa codebase.


# License

This software is licensed and distributed under the same terms as [Ixa](https://github.com/CDCgov/ixa). See https://github.com/CDCgov/ixa#public-domain-standard-notice.
