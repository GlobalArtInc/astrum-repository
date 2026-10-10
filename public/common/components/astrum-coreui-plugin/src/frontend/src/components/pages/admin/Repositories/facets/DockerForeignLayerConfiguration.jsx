

import React, {useState} from 'react';

import {FormUtils, ValidationUtils} from '@globalart/astrum-ui-plugin';
import {isEmpty} from 'ramda';

import {faTrashAlt, faPlusCircle} from '@fortawesome/free-solid-svg-icons';

import {
  AsCheckbox,
  AsFieldset,
  AsFormGroup,
  AsTextInput,
  AsList,
  AsFontAwesomeIcon,
  AsButton,
  AsFormRow,
} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

export default function DockerForeignLayerConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  const {dockerProxy} = currentParent.context.data;

  const cacheForeignLayers = dockerProxy?.cacheForeignLayers || false;
  const foreignLayerUrlWhitelist = dockerProxy?.foreignLayerUrlWhitelist || [];

  const [prevWhitelist, setWhitelist] = useState(
    isEmpty(foreignLayerUrlWhitelist) ? ['.*'] : foreignLayerUrlWhitelist
  );

  const updateForeignLayerUrlWhiteList = (value) => {
    sendParent({
      type: 'UPDATE',
      name: 'dockerProxy.foreignLayerUrlWhitelist',
      value,
    });
  };

  const toggleForeignLayer = () => {
    let whiteList = foreignLayerUrlWhitelist;
    const cacheForeignLayers = !dockerProxy.cacheForeignLayers;

    whiteList = cacheForeignLayers ? prevWhitelist : [];

    sendParent({
      type: 'UPDATE',
      name: 'dockerProxy.cacheForeignLayers',
      value: cacheForeignLayers,
    });

    updateForeignLayerUrlWhiteList(whiteList);
  };

  const addUrlPattern = () => {
    const current = dockerProxy.foreignLayerUrlWhitelist || [];
    const foreignLayer = dockerProxy?.foreignLayer || '';
    const existLayer = current.includes(foreignLayer);
    const isValidLayer = !existLayer && !ValidationUtils.isBlank(foreignLayer);
    const whiteList = isValidLayer ? [...current, foreignLayer] : current;

    sendParent({
      type: 'UPDATE',
      name: 'dockerProxy.foreignLayer',
      value: '',
    });

    if (isValidLayer) {
      updateForeignLayerUrlWhiteList(whiteList);
      setWhitelist(whiteList);
    }
  };

  const removeUrlPattern = (index) => {
    if (foreignLayerUrlWhitelist.length > 1) {
      const whiteList = dockerProxy.foreignLayerUrlWhitelist.filter(
          (_, i) => i !== index
      );

      updateForeignLayerUrlWhiteList(whiteList);
      setWhitelist(whiteList);
    }
  };

  const handleEnter = (event) => {
    if (event.key === 'Enter') {
      event.preventDefault();
      addUrlPattern();
    }
  };

  return (
    <>
      <AsFieldset label={EDITOR.FOREIGN_LAYER.CACHING} isRequired>
        <AsCheckbox
          {...FormUtils.checkboxProps(
            'dockerProxy.cacheForeignLayers',
            currentParent
          )}
          onChange={toggleForeignLayer}
        >
          {EDITOR.FOREIGN_LAYER.CHECKBOX}
        </AsCheckbox>
      </AsFieldset>
      {cacheForeignLayers && (
        <AsFormRow className="as-foreign-layer">
          <>
            <AsFormGroup
              label={EDITOR.FOREIGN_LAYER.URL}
              sublabel={EDITOR.FOREIGN_LAYER.URL_SUBLABEL}
              isRequired
            >
              <AsTextInput
                className="as-text-input--long"
                {...FormUtils.fieldProps(
                  'dockerProxy.foreignLayer',
                  currentParent
                )}
                onChange={FormUtils.handleUpdate(
                  'dockerProxy.foreignLayer',
                  sendParent
                )}
                onKeyDown={handleEnter}
              />
            </AsFormGroup>
            <AsButton
              variant="icon-only"
              title={EDITOR.FOREIGN_LAYER.ADD}
              onClick={addUrlPattern}
              type="button"
            >
              <AsFontAwesomeIcon icon={faPlusCircle} />
            </AsButton>
          </>
        </AsFormRow>
      )}
      {cacheForeignLayers && foreignLayerUrlWhitelist.length > 0 && (
        <AsList className="as-foreign-layer">
          {foreignLayerUrlWhitelist.map((layer, index) => (
            <AsList.Item key={layer}>
              <AsList.Text>{layer}</AsList.Text>
              <AsList.Actions>
                <AsButton
                  title={EDITOR.FOREIGN_LAYER.REMOVE}
                  variant="icon-only"
                  onClick={() => removeUrlPattern(index)}
                  className={foreignLayerUrlWhitelist.length <= 1 ? 'disabled' : null}
                  type="button"
                >
                  <AsFontAwesomeIcon icon={faTrashAlt} />
                </AsButton>
              </AsList.Actions>
            </AsList.Item>
          ))}
        </AsList>
      )}
    </>
  );
}
