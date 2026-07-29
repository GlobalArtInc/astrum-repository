
import {filter, keys} from "ramda";
import ValidationUtils from '../../../interface/ValidationUtils';

import TextFieldFactory from './factory/TextFieldFactory';
import ComboboxFieldFactory from './factory/ComboboxFieldFactory';
import SetOfCheckboxesFieldFactory from './factory/SetOfCheckboxesFieldFactory';

import {SUPPORTED_FIELD_TYPES} from './FormFieldsFactoryConstants';

export default class FormFieldsFactory {
  static ALL_FIELDS = [
      TextFieldFactory,
      ComboboxFieldFactory,
      SetOfCheckboxesFieldFactory
  ];

  static getFields(fields) {
    return fields.map(props => {
      const field = FormFieldsFactory.ALL_FIELDS.find(field => field.types.includes(props.type));
      return {
        Field: field.component,
        props,
      };
    });
  }

  static defaultValues(type, types) {
    const fields = types[type]?.formFields;
    const result = {};
    let value = null;
    fields?.forEach(({id, type, initialValue}) => {
      if (initialValue) {
        value = initialValue;
      } else if (
          SUPPORTED_FIELD_TYPES.TEXT.includes(type) ||
          SUPPORTED_FIELD_TYPES.COMBOBOX.includes(type)
      ) {
        value = '';
      } else if (
          SUPPORTED_FIELD_TYPES.SET_OF_CHECKBOXES.includes(type)
      ) {
        value = {};
      }
      result[id] = value;
    });

    return result;
  }

  static getValidations(data, types) {
    const fields = types[data.type]?.formFields;
    const errors = {};

    fields?.forEach(({id, required}) => {
      if (required) {
        if (id === 'actions') {
          errors[id] = ValidationUtils.validateNotBlank(keys(filter(v => v === true, data[id])).join());
        } else {
          errors[id] = ValidationUtils.validateNotBlank(data[id]);
        }
      }
    });
    return errors;
  }
}
