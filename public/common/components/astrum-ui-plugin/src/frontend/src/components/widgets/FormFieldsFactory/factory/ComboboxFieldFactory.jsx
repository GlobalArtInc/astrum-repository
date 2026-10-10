

import React from 'react';
import {assign, actions, send, createMachine} from 'xstate';
import {useMachine} from '@xstate/react';

import {
  AsFormSelect,
  AsCombobox,
} from '@globalart/react-shared-components';

import FormUtils from '../../../../interface/FormUtils';
import ExtAPIUtils from '../../../../interface/ExtAPIUtils';

import {SUPPORTED_FIELD_TYPES} from '../FormFieldsFactoryConstants';
import UIStrings from '../../../../constants/UIStrings';
import APIConstants from '../../../../constants/APIConstants';

const Field = ({id, dynamicProps, current:parentState, onChange}) => {
  const {storeApi, allowAutocomplete, initialValue, idMapping, nameMapping, readOnly} = dynamicProps;
  if (typeof storeApi !== 'string') {
    throw new Error('storeApi must be a string');
  }
  const [action, method] = storeApi.split('.');
  const value = parentState.context.data[id] || '';
  const loadOnInit = !allowAutocomplete || value;

  const [state, sendEvent] = useMachine(() =>
      createMachine({
        id: 'ComboboxMachine',
        initial: loadOnInit ? 'fetching' : 'loaded',
        states: {
          fetching: {
            invoke: {
              src: 'fetch',
              onDone: {
                target: 'loaded',
                actions: ['setData'],
              },
              onError: {
                target: 'error',
                actions: ['setError'],
              }
            }
          },
          loading: {},
          loaded: {},
          error: {},
        },
        on: {
          LOAD: {
            target: 'loading',
          },
          FETCH: {
            target: 'fetching',
          },
          LOAD_WITH_DEBOUNCE: {
            target: 'loading',
            actions: ['setQuery', 'setAction', 'setMethod', 'resetError', 'debounceApiCall', 'doFetch'],
          },
          RESET_MATCHES: {
            target: 'loaded',
            actions: ['debounceApiCall', 'resetMatches'],
          },
        }
      }, {
        actions: {
          setData: assign({
            data: (_, {data}) => {
              return data?.map(item => ({
                id: item[idMapping || 'id'],
                displayName: item[nameMapping || 'name']
              }));
            }
          }),
          setQuery: assign({
            query: (_, {query}) => query,
          }),
          setAction: assign({
            action: (_, {action}) => action,
          }),
          setMethod: assign({
            method: (_, {method}) => method,
          }),
          setError: assign({
            error: (_, event) => event?.data?.message || UIStrings.ERROR.UNKNOWN
          }),
          resetMatches: assign({
            data: [],
          }),
          resetError: assign({
            error: null,
          }),
          debounceApiCall: actions.cancel('debounced-api-call'),
          doFetch: send({
            type: 'FETCH',
            id: 'debounced-api-call',
            delay: APIConstants.DEBOUNCE_DELAY,
          }),
        },
        services: {
          fetch: async ({query, action, method}) => {
            const options = allowAutocomplete ? {query} : null;
            const response = await ExtAPIUtils.extAPIRequest(action, method, options);
            ExtAPIUtils.checkForError(response);
            return ExtAPIUtils.extractResult(response, []);
          }
        }
      }), {
        context: {
          query: value,
          action,
          method,
          data: [],
        },
        devTools: true,
      },
  );

  const {data, error} = state.context;
  const selectedItem = data.find(item => item.id === value);
  const isLoading = state.matches('loading') || state.matches('fetching');

  const loadData = (query) => {
    if (query) {
      sendEvent({type: 'LOAD_WITH_DEBOUNCE', query, action, method});
    } else {
      sendEvent({type: 'RESET_MATCHES'});
    }
  };

  const onChangeCombobox = (query, dataItem) => onChange(id, dataItem?.id || query)

  if (readOnly) {
    return selectedItem?.displayName || value;
  }

  return <>
    {allowAutocomplete
        ? <AsCombobox
            {...FormUtils.fieldProps(id, parentState)}
            value={selectedItem?.displayName || value}
            onChange={onChangeCombobox}
            onSearch={loadData}
            loading={isLoading}
            autoComplete={false}
            matches={data}
            loadError={error}
            aria-label="combobox"
          />
        : <AsFormSelect
            {...FormUtils.fieldProps(id, parentState, initialValue || '')}
            value={value}
            onChange={value => onChange(id, value)}
        >
          <option value=""/>
          {data?.map(({id, displayName}) => <option key={id} value={id}>{displayName}</option>)}
        </AsFormSelect>
    }
  </>;
}

export default {
  types: SUPPORTED_FIELD_TYPES.COMBOBOX,
  component: Field,
};
