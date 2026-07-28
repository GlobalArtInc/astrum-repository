

import { assign } from 'xstate';
import ExtAPIUtils from '../../../interface/ExtAPIUtils';
import APIConstants from '../../../constants/APIConstants';
import FormUtils from '../../../interface/FormUtils';
import ValidationUtils from '../../../interface/ValidationUtils';
import UIStrings from '../../../constants/UIStrings';

const { ACTION, METHODS } = APIConstants.EXT.CAPABILITY;

// Helper functions to calculate field-related flags
const calculateHasFields = selectedType => {
  return selectedType?.formFields && selectedType.formFields.length > 0;
};

const calculateAllFieldsOptional = selectedType => {
  const hasFields = calculateHasFields(selectedType);
  if (!hasFields) {
    return false;
  }
  // A field blocks pristine saves only if it's required AND not read-only
  // Read-only required fields don't count since user can't change them anyway
  return selectedType.formFields.every(field => field.required !== true || field.readOnly === true);
};

export default FormUtils.buildFormMachine({
  id: 'CapabilitiesCreateMachine',
  config: config => ({
    ...config,
    context: {
      ...config.context,
      types: [],
      selectedType: null,
      submitAttempted: false,
      shouldShowErrors: false,
      hasFields: false,
      allFieldsOptional: false,
    },
    states: {
      ...config.states,
      loaded: {
        ...config.states.loaded,
        on: {
          ...config.states.loaded.on,
          SET_SELECTED_TYPE: {
            target: 'loaded',
            actions: ['setSelectedType', 'initializeDataFromInitialValues', 'validate'],
          },
          SAVE: [
            {
              target: 'saving',
              cond: 'canSave',
            },
            {
              target: 'loaded',
              actions: ['touchAllFields', 'setSubmitAttempted', 'validate'],
            },
          ],
        },
      },
    },
  }),
}).withConfig({
  actions: {
    setData: assign({
      data: (_, event) => event.data?.data || {},
      pristineData: (_, event) => event.data?.data || {},
      types: (_, event) => event.data || [],
    }),
    resetToPristine: assign({
      data: () => ({}),
      pristineData: () => ({}),
      isTouched: () => ({}),
      validationErrors: () => ({}),
      saveErrors: () => ({}),
      saveErrorData: () => ({}),
      saveError: () => undefined,
      isPristine: () => true,
    }),
    setSelectedType: assign({
      selectedType: (_, event) => event.selectedType || null,
      hasFields: (_, event) => calculateHasFields(event.selectedType),
      allFieldsOptional: (_, event) => calculateAllFieldsOptional(event.selectedType),
    }),
    initializeDataFromInitialValues: assign(({ selectedType }) => {
      const data = {};
      const pristineData = {};

      if (selectedType?.formFields) {
        selectedType.formFields.forEach(field => {
          if (field.initialValue !== undefined && field.initialValue !== null) {
            data[field.id] = field.initialValue;
            pristineData[field.id] = field.initialValue;
          }
        });
      }

      return {
        data,
        pristineData,
        isTouched: {},
        validationErrors: {},
        saveErrors: {},
        saveErrorData: {},
        saveError: undefined,
        isPristine: true,
        submitAttempted: false,
        shouldShowErrors: false,
        hasFields: calculateHasFields(selectedType),
        allFieldsOptional: calculateAllFieldsOptional(selectedType),
      };
    }),
    validate: assign(({ data, selectedType, isTouched, submitAttempted }) => {
      const fullErrors = {};

      if (!selectedType) {
        return {
          validationErrors: {},
          shouldShowErrors: false,
        };
      }

      const fields = selectedType.formFields || [];
      // Get the field IDs for the current type to filter touched state
      const currentFieldIds = new Set(fields.map(f => f.id));

      fields.forEach(field => {
        const value = data?.[field.id];

        if (field.required) {
          if (field.type === 'itemselect') {
            if (!value || (Array.isArray(value) && value.length === 0)) {
              fullErrors[field.id] = UIStrings.ERROR.FIELD_REQUIRED;
              return;
            }
          } else if (ValidationUtils.isBlank(value)) {
            fullErrors[field.id] = UIStrings.ERROR.FIELD_REQUIRED;
            return;
          }
        }

        if (field.type === 'number' && value !== null && value !== undefined && value !== '') {
          const min = field.minValue ?? Number.NEGATIVE_INFINITY;
          const max = field.maxValue ?? Number.POSITIVE_INFINITY;
          const numberError = ValidationUtils.isInRange({ value, min, max, allowDecimals: false });
          if (numberError) {
            fullErrors[field.id] = numberError;
            return;
          }
        }

        if (field.type === 'url' && value) {
          const urlError = ValidationUtils.validateIsUrl(value);
          if (urlError) {
            fullErrors[field.id] = urlError;
            return;
          }
        }
      });

      // Only show errors for touched fields that belong to the current type
      // This prevents showing errors from previous type selections
      const visibleErrors = {};
      Object.entries(fullErrors).forEach(([key, value]) => {
        if (currentFieldIds.has(key) && isTouched?.[key]) {
          visibleErrors[key] = value;
        }
      });

      // Calculate shouldShowErrors based on the newly computed visibleErrors
      const isInvalid = FormUtils.isInvalid(visibleErrors);
      const shouldShowErrors = submitAttempted && isInvalid;

      return {
        validationErrors: visibleErrors,
        shouldShowErrors,
      };
    }),
    touchAllFields: assign(({ selectedType, isTouched = {} }) => {
      const updatedTouched = { ...isTouched };
      const fields = selectedType?.formFields || [];
      fields.forEach(f => {
        updatedTouched[f.id] = true;
      });
      return { isTouched: updatedTouched };
    }),
    onSaveSuccess: () => {
      // Navigation handled in component
    },
    setSubmitAttempted: assign({
      submitAttempted: () => true,
    }),
  },

  guards: {
    canSave: ({ isPristine, validationErrors, selectedType, data, hasFields, allFieldsOptional }) => {
      // Validate all required fields directly (not just touched ones)
      // This prevents submission when required fields are empty, even if not touched
      let hasValidationErrors = false;
      if (hasFields && selectedType?.formFields) {
        for (const field of selectedType.formFields) {
          if (field.required) {
            const value = data?.[field.id];
            if (field.type === 'itemselect') {
              if (!value || (Array.isArray(value) && value.length === 0)) {
                hasValidationErrors = true;
                break;
              }
            } else if (ValidationUtils.isBlank(value)) {
              hasValidationErrors = true;
              break;
            }
          }
        }
      }

      // Also check existing validation errors (for touched fields)
      const isValid = !FormUtils.isInvalid(validationErrors) && !hasValidationErrors;

      // For forms with no fields (like audit), allow saving as long as type is selected and form is valid
      if (!hasFields && selectedType) {
        return isValid;
      }

      // For forms with all optional/read-only fields, allow saving if valid (even if pristine)
      if (allFieldsOptional) {
        return isValid;
      }

      // Check if all required fields have values (including prefilled initial values)
      // If form is valid and all required fields are satisfied, allow pristine saves
      // This handles cases where required fields are prefilled with initial values
      const allRequiredFieldsHaveValues =
        hasFields &&
        selectedType.formFields
          .filter(field => field.required === true && field.readOnly !== true)
          .every(field => {
            const value = data?.[field.id];
            if (field.type === 'itemselect') {
              return value && (Array.isArray(value) ? value.length > 0 : true);
            }
            return !ValidationUtils.isBlank(value);
          });

      // If form is valid and all required fields are satisfied (even if pristine), allow save
      if (isValid && allRequiredFieldsHaveValues) {
        return true;
      }

      // Otherwise, require form to be dirty and valid
      return !isPristine && isValid;
    },
  },

  services: {
    fetchData: () => ExtAPIUtils.extAPIRequest(ACTION, METHODS.READ_TYPES).then(ExtAPIUtils.checkForErrorAndExtract),
    saveData: ({ data, selectedType }) => {
      // Transform properties to match ExtJS format
      // itemselect fields need to be converted from arrays to strings
      const transformedProperties = { ...data };
      if (selectedType?.formFields) {
        selectedType.formFields.forEach(field => {
          if (field.type === 'itemselect' && transformedProperties[field.id] !== undefined) {
            const value = transformedProperties[field.id];
            if (Array.isArray(value)) {
              // Convert array to string - join with comma if multiple items, or use single item
              // This matches ExtJS ItemSelector behavior when valueAsString is true
              transformedProperties[field.id] = value.join(',');
            }
          }
        });
      }

      const payload = {
        typeId: selectedType?.id,
        enabled: true,
        notes: transformedProperties?.notes || '',
        properties: transformedProperties,
      };

      // Remove notes from properties if it exists there (it should be at the top level)
      delete payload.properties.notes;

      // ExtJS Direct API expects data as an array
      return ExtAPIUtils.extAPIRequest(ACTION, METHODS.CREATE, { data: [payload] }).then(response => {
        ExtAPIUtils.checkForError(response);
        return ExtAPIUtils.extractResult(response, {});
      });
    },
  },
});
