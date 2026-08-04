
import FormFieldsFactory from './FormFieldsFactory';
import TextFieldFactory from './factory/TextFieldFactory';
import ComboboxFieldFactory from './factory/ComboboxFieldFactory';
import SetOfCheckboxesFieldFactory from './factory/SetOfCheckboxesFieldFactory';

describe('FormFieldsFactory', () => {
  describe('getFields', () => {
    it('maps text field to TextFieldFactory component', () => {
      const fields = [{type: 'textfield', id: 'name'}];

      const result = FormFieldsFactory.getFields(fields);

      expect(result).toHaveLength(1);
      expect(result[0].Field).toBe(TextFieldFactory.component);
      expect(result[0].props).toEqual({type: 'textfield', id: 'name'});
    });

    it('maps string field to TextFieldFactory component', () => {
      const fields = [{type: 'string', id: 'value'}];

      const result = FormFieldsFactory.getFields(fields);

      expect(result[0].Field).toBe(TextFieldFactory.component);
    });

    it('maps password field to TextFieldFactory component', () => {
      const fields = [{type: 'password', id: 'secret'}];

      const result = FormFieldsFactory.getFields(fields);

      expect(result[0].Field).toBe(TextFieldFactory.component);
    });

    it('maps combobox field to ComboboxFieldFactory component', () => {
      const fields = [{type: 'combobox', id: 'selector'}];

      const result = FormFieldsFactory.getFields(fields);

      expect(result[0].Field).toBe(ComboboxFieldFactory.component);
    });

    it('maps setOfCheckboxes field to SetOfCheckboxesFieldFactory component', () => {
      const fields = [{type: 'setOfCheckboxes', id: 'options'}];

      const result = FormFieldsFactory.getFields(fields);

      expect(result[0].Field).toBe(SetOfCheckboxesFieldFactory.component);
    });

    it('handles multiple fields of different types', () => {
      const fields = [
        {type: 'textfield', id: 'name'},
        {type: 'combobox', id: 'category'},
        {type: 'password', id: 'token'}
      ];

      const result = FormFieldsFactory.getFields(fields);

      expect(result).toHaveLength(3);
      expect(result[0].Field).toBe(TextFieldFactory.component);
      expect(result[1].Field).toBe(ComboboxFieldFactory.component);
      expect(result[2].Field).toBe(TextFieldFactory.component);
    });
  });

  describe('defaultValues', () => {
    it('returns empty object when no formFields defined', () => {
      const types = {myType: {}};

      const result = FormFieldsFactory.defaultValues('myType', types);

      expect(result).toEqual({});
    });

    it('returns empty object when type does not exist', () => {
      const types = {};

      const result = FormFieldsFactory.defaultValues('nonexistent', types);

      expect(result).toEqual({});
    });

    it('uses initialValue when provided for text field', () => {
      const types = {
        myType: {
          formFields: [
            {id: 'name', type: 'textfield', initialValue: 'default name'}
          ]
        }
      };

      const result = FormFieldsFactory.defaultValues('myType', types);

      expect(result).toEqual({name: 'default name'});
    });

    it('uses empty string as default for text field without initialValue', () => {
      const types = {
        myType: {
          formFields: [
            {id: 'name', type: 'textfield'}
          ]
        }
      };

      const result = FormFieldsFactory.defaultValues('myType', types);

      expect(result).toEqual({name: ''});
    });

    it('uses empty string as default for string field', () => {
      const types = {
        myType: {
          formFields: [
            {id: 'value', type: 'string'}
          ]
        }
      };

      const result = FormFieldsFactory.defaultValues('myType', types);

      expect(result).toEqual({value: ''});
    });

    it('uses empty string as default for password field', () => {
      const types = {
        myType: {
          formFields: [
            {id: 'secret', type: 'password'}
          ]
        }
      };

      const result = FormFieldsFactory.defaultValues('myType', types);

      expect(result).toEqual({secret: ''});
    });

    it('uses empty string as default for combobox field', () => {
      const types = {
        myType: {
          formFields: [
            {id: 'selector', type: 'combobox'}
          ]
        }
      };

      const result = FormFieldsFactory.defaultValues('myType', types);

      expect(result).toEqual({selector: ''});
    });

    it('uses empty object as default for setOfCheckboxes field', () => {
      const types = {
        myType: {
          formFields: [
            {id: 'options', type: 'setOfCheckboxes'}
          ]
        }
      };

      const result = FormFieldsFactory.defaultValues('myType', types);

      expect(result).toEqual({options: {}});
    });

    it('handles multiple fields with mixed defaults', () => {
      const types = {
        myType: {
          formFields: [
            {id: 'name', type: 'textfield', initialValue: 'John'},
            {id: 'password', type: 'password'},
            {id: 'category', type: 'combobox'},
            {id: 'permissions', type: 'setOfCheckboxes'}
          ]
        }
      };

      const result = FormFieldsFactory.defaultValues('myType', types);

      expect(result).toEqual({
        name: 'John',
        password: '',
        category: '',
        permissions: {}
      });
    });
  });

  describe('getValidations', () => {
    it('returns empty object when no formFields defined', () => {
      const data = {type: 'myType'};
      const types = {myType: {}};

      const result = FormFieldsFactory.getValidations(data, types);

      expect(result).toEqual({});
    });

    it('returns empty object when type does not exist', () => {
      const data = {type: 'nonexistent'};
      const types = {};

      const result = FormFieldsFactory.getValidations(data, types);

      expect(result).toEqual({});
    });

    it('does not validate optional fields', () => {
      const data = {type: 'myType', name: ''};
      const types = {
        myType: {
          formFields: [
            {id: 'name', required: false}
          ]
        }
      };

      const result = FormFieldsFactory.getValidations(data, types);

      expect(result).toEqual({});
    });

    it('validates required field with value as valid', () => {
      const data = {type: 'myType', name: 'value'};
      const types = {
        myType: {
          formFields: [
            {id: 'name', required: true}
          ]
        }
      };

      const result = FormFieldsFactory.getValidations(data, types);

      expect(result.name).toBeUndefined();
    });

    it('validates empty required field as invalid', () => {
      const data = {type: 'myType', name: ''};
      const types = {
        myType: {
          formFields: [
            {id: 'name', required: true}
          ]
        }
      };

      const result = FormFieldsFactory.getValidations(data, types);

      expect(result.name).toBeTruthy();
      expect(typeof result.name).toBe('string');
    });

    it('validates multiple required fields', () => {
      const data = {type: 'myType', name: 'value', password: ''};
      const types = {
        myType: {
          formFields: [
            {id: 'name', required: true},
            {id: 'password', required: true}
          ]
        }
      };

      const result = FormFieldsFactory.getValidations(data, types);

      expect(result.name).toBeUndefined();
      expect(result.password).toBeTruthy();
    });

    it('validates actions field with selections as valid', () => {
      const data = {
        type: 'myType',
        actions: {read: true, write: false, delete: true}
      };
      const types = {
        myType: {
          formFields: [
            {id: 'actions', required: true}
          ]
        }
      };

      const result = FormFieldsFactory.getValidations(data, types);

      // Should be valid when at least one action is selected
      expect(result.actions).toBeUndefined();
    });

    it('validates actions field with no selections as invalid', () => {
      const data = {
        type: 'myType',
        actions: {read: false, write: false}
      };
      const types = {
        myType: {
          formFields: [
            {id: 'actions', required: true}
          ]
        }
      };

      const result = FormFieldsFactory.getValidations(data, types);

      // Should be invalid when no actions are selected
      expect(result.actions).toBeTruthy();
    });
  });
});
