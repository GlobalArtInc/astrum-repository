

import { assign } from 'xstate';
import { restClient, ENDPOINTS, parseApiError } from '../../../../../../interface/api';
import { createFormMachine, type FormContext, type ValidationErrors } from '../../../../../../interface/form';
import { JdbcParameter } from './JdbcParameterEditor';
import {
  DataStoreConfig,
  validateConnectionPool,
  parseAdvancedString,
  serializeParameters,
} from './types';

export interface DataStoreFormData {
  maximumConnectionPool: number | string;
  jdbcParameters: JdbcParameter[];
  jdbcUrl: string;
  username: string;
  schema: string;
}

export const DEFAULT_DATASTORE_FORM_DATA: DataStoreFormData = {
  maximumConnectionPool: 10,
  jdbcParameters: [],
  jdbcUrl: '',
  username: '',
  schema: '',
};

/** REST config → form data. Parses `advanced` exactly once so parameter ids stay
 *  stable and the factory's deep-equality dirty check matches serialize comparison. */
export function toFormData(config: DataStoreConfig): DataStoreFormData {
  return {
    maximumConnectionPool: config?.maximumConnectionPool ?? 10,
    jdbcParameters: parseAdvancedString(config?.advanced || ''),
    jdbcUrl: config?.jdbcUrl || '',
    username: config?.username || '',
    schema: config?.schema || '',
  };
}

/** Form data → PUT payload (only the two editable fields, matching today). */
export function toUpdatePayload(data: DataStoreFormData): { maximumConnectionPool: number; advanced: string } {
  return {
    maximumConnectionPool: parseInt(String(data.maximumConnectionPool), 10) || 10,
    advanced: serializeParameters(data.jdbcParameters) || '',
  };
}

export function createDataStoreFormMachine() {
  return createFormMachine<DataStoreFormData>({
    id: 'datastore-form',
    stayEditableAfterSave: true,
    context: {
      data: { ...DEFAULT_DATASTORE_FORM_DATA },
    },
    actions: {
      validate: assign((ctx: FormContext<DataStoreFormData>) => {
        const errors: ValidationErrors = {};
        const poolError = validateConnectionPool(ctx.data.maximumConnectionPool);
        if (poolError) {
          errors.maximumConnectionPool = poolError;
        }
        return { validationErrors: errors };
      }),
      // Factory default onSaveSuccess sets pristineData = data and ignores the save
      // result. DataStore keeps local params but refreshes pool from the server result.
      onSaveSuccess: assign((ctx: FormContext<DataStoreFormData>, event: unknown) => {
        const saved = (event as { data?: DataStoreConfig })?.data;
        const data: DataStoreFormData = {
          ...ctx.data,
          maximumConnectionPool: saved?.maximumConnectionPool ?? ctx.data.maximumConnectionPool,
        };
        return { data, pristineData: data, isPristine: true, touched: {} };
      }),
    },
    services: {
      load: async () => {
        const config = await restClient.get<DataStoreConfig>(ENDPOINTS.DATASTORE);
        return { data: toFormData(config ?? ({} as DataStoreConfig)) };
      },
      // Base save used by machine tests; the hook overrides this to add toast + humanized errors.
      save: async (ctx: FormContext<DataStoreFormData>) => {
        try {
          return await restClient.put<DataStoreConfig>(ENDPOINTS.DATASTORE, toUpdatePayload(ctx.data));
        } catch (err) {
          throw new Error(parseApiError(err).message);
        }
      },
    },
  });
}
