/*!



*/

use std::any::{Any, TypeId};
use std::collections::{HashMap};
use crate::any_map::AnyMap;
use crate::{New, PersonId, IxaError};
use crate::context::Context;
use crate::init_list::InitializationList;
use crate::property::Property;
use crate::query::Query;

pub(crate) struct PeopleData {
  // pub(super) is_initializing: bool,
  pub(super) current_population: usize,
  pub(crate) properties_map: AnyMap,
  pub(crate) registered_derived_properties: Vec<TypeId>,
  pub(crate) dependency_map: HashMap<TypeId, Vec<TypeId>>,
  // This is actually a `HashMap<TypeId, Index>`
  pub(super) property_indexes: HashMap<TypeId, Box<dyn Any>>,
  // pub(super) people_types: RefCell<HashMap<String, TypeId>>,
}

impl Default for PeopleData {
  fn default() -> Self {
    PeopleData{
      current_population: 0,
      properties_map: AnyMap::new(),
      registered_derived_properties: vec![],
      dependency_map: HashMap::new(),
      property_indexes: Default::default(),
    }
  }
}

impl New for PeopleData{
  const new: &'static dyn Fn() -> Self = &PeopleData::default;
}

impl PeopleData {
  pub fn create_population(&mut self, size: usize) {
    self.current_population = size;
  }

  pub fn add_person(&mut self) -> PersonId{
    let person_id = PersonId(self.current_population);
    self.current_population += 1;
    person_id
  }

  pub fn get_person_property_ref<T: Property>(&self, person_id: PersonId) -> &Option<T> {
    let idx = person_id.0;
    let property_values: &Vec<Option<T>> = self.properties_map.get_vec_ref_unchecked();

    if idx >= property_values.len() {
      &None
    } else {
      &property_values[idx]
    }

  }

  pub fn get_person_property_mut<T: Property>(&mut self, person_id: PersonId) -> &mut Option<T> {
    let idx = person_id.0;
    let property_values: &mut Vec<Option<T>> = self.properties_map.get_vec_mut();

    if idx >= property_values.len() {
      property_values.resize_with(idx + 1, || None);
    }

    &mut property_values[idx]
  }

  pub fn set_property<T: Property>(&mut self, person_id: PersonId, value: T) {
    let property = self.get_person_property_mut(person_id);
    *property = Some(value);
  }

  pub fn check_initialization_list(&self, list: &dyn InitializationList) -> Result<(), ()>{
    Ok(())
  }
}


pub trait ContextPeopleExt {
  fn get_current_population(&self) -> usize;
  fn add_person<T: InitializationList>(&mut self, properties: T) -> Result<PersonId, IxaError>;
  fn get_person_property<T: Property>(&self, person_id: PersonId) -> &Option<T>;
  fn get_person_property_mut<T: Property>(&mut self, person_id: PersonId) -> &mut Option<T>;
  fn get_person_property_or_default<T: Property>(&mut self, person_id: PersonId, default: T) -> &mut T;
  fn set_person_property<T: Property>(&mut self, person_id: PersonId, value: T);
  fn query_people<T: Query>(&self, q: T) -> Vec<PersonId>;
}

impl ContextPeopleExt for Context {
  fn get_current_population(&self) -> usize {
    match self.get_data_container::<PeopleData>() {
      None => 0,
      Some(people_data) => {
        people_data.current_population
      }
    }
  }

  /// Adds a new person with the given list of properties.
  fn add_person<T: InitializationList>(&mut self, properties: T) -> Result<PersonId, IxaError> {
    let people_data = self.get_data_container_mut::<PeopleData>();
    people_data.check_initialization_list(&properties)?;

    let person_id = people_data.add_person();

    // Initialize the properties. We set |is_initializing| to prevent
    // set_property() from generating an event.
    // people_data.is_initializing = true;
    properties.set_properties(people_data, person_id);
    // people_data.is_initializing = false;

    Ok(person_id)
  }

  /// Gets a copy of the value of the property for the given person.
  fn get_person_property<T: Property>(&self, person_id: PersonId) -> &Option<T> {
    self.get_data_container::<PeopleData>()
        .unwrap()
        .get_person_property_ref(person_id)
  }
  
  /// Gets a copy of the value of the property for the given person.
  fn get_person_property_mut<T: Property>(&mut self, person_id: PersonId) -> &mut Option<T> {
    self.get_data_container_mut::<PeopleData>()
        .get_person_property_mut(person_id)
  }

  /// Gets a copy of the value of the property for the given person if it
  /// exists, or else sets the property to the default value and returns that.
  // ToDo: Does not emit event (or respect `PeopleData::is_initializing`)
  fn get_person_property_or_default<T: Property>(&mut self, person_id: PersonId, default: T) -> &mut T {
    let property: &mut Option<T> = self.get_data_container_mut::<PeopleData>()
                          .get_person_property_mut(person_id);
    
    match property {
      Some(value) => value,
      
      None => {
        *property = Some(default);
        property.as_mut().unwrap()
      }
    }
  }
  
  fn set_person_property<T: Property>(&mut self, person_id: PersonId, value: T) {
    let property: &mut Option<T> = self.get_data_container_mut::<PeopleData>()
                                       .get_person_property_mut(person_id);
    *property = Some(value);
  }

  fn query_people<T: Query>(&self, q: T) -> Vec<PersonId> {
    
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Clone, Eq, PartialEq, Debug)]
  struct Age(u8);
  impl Property for Age {}

  #[derive(Clone, Eq, PartialEq, Debug)]
  struct Name(String);
  impl Property for Name {}

  #[derive(Clone, Eq, PartialEq, Debug)]
  enum InfectionStatus {
    I,
    R,
    S
  }
  impl Property for InfectionStatus {}

  #[test]
  fn test_people_data() {
    let mut context = Context::new();

    context.add_person((Age(10), Name("John Smith".to_string()), InfectionStatus::I));



  }
}
