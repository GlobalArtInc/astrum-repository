
import React from 'react';

import {FormUtils, useSimpleMachine} from '@globalart/astrum-ui-plugin';

import {AsFormGroup, AsFormSelect, AsLoadWrapper} from '@globalart/astrum-react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

export const RECIPES_URL = 'service/rest/internal/ui/repositories/recipes';

export default function GenericFormatConfiguration({parentMachine}) {
  const {current, retry, isLoading} = useSimpleMachine({
    id: 'GenericFormatConfigurationMachine',
    url: RECIPES_URL
  });

  const [currentParent, sendParent] = parentMachine;

  const {
    pristineData: {name}
  } = currentParent.context;

  const isEdit = !!name;

  const {data: recipes, error} = current.context;

  const formats = getFormats(recipes);

  const types = getTypes(recipes);

  const {format, type} = currentParent.context.data;

  const handleFormatUpdate = (format) => {
    const repoType = types.get(format)?.includes(type) ? type : '';
    sendParent({type: 'RESET_DATA', format, repoType});
  };

  const handleTypeUpdate = (repoType) => {
    sendParent({type: 'RESET_DATA', format, repoType});
  };

  return (
    <>
      <h2 className="as-h2">{EDITOR.FORMAT_AND_TYPE_CAPTION}</h2>
      <AsLoadWrapper loading={isLoading} error={error} retryHandler={retry}>
        <div className="as-form-row">
          <AsFormGroup label={EDITOR.FORMAT_LABEL} isRequired className="as-form-group-format">
            <AsFormSelect
              {...FormUtils.selectProps('format', currentParent)}
              onChange={handleFormatUpdate}
              disabled={isEdit}
            >
              <option value="">{EDITOR.SELECT_FORMAT_OPTION}</option>
              {formats?.map((format) => (
                <option key={format} value={format}>
                  {format}
                </option>
              ))}
            </AsFormSelect>
          </AsFormGroup>

          <AsFormGroup label={EDITOR.TYPE_LABEL} isRequired className="as-form-group-type">
            <AsFormSelect
              {...FormUtils.selectProps('type', currentParent)}
              onChange={handleTypeUpdate}
              disabled={!format || isEdit}
            >
              <option value="">{EDITOR.SELECT_TYPE_OPTION}</option>
              {types?.get(format)?.map((type) => (
                <option key={type} value={type}>
                  {type}
                </option>
              ))}
            </AsFormSelect>
          </AsFormGroup>
        </div>
      </AsLoadWrapper>
    </>
  );
}

const getFormats = (recipes) => [...new Set(recipes?.map((r) => r.format))].sort();

const getTypes = (recipes) =>
  recipes?.reduce((acc, curr) => {
    const {format, type} = curr;
    acc.has(format) ? acc.get(format).push(type) : acc.set(format, [type]);
    return acc;
  }, new Map());
