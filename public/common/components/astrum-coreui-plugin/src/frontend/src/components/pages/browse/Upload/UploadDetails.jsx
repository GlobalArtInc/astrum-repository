
import React, { useCallback } from 'react';
import { useMachine } from '@xstate/react';
import {
  NxButton,
  NxButtonBar,
  NxCheckbox,
  NxFileUpload,
  NxFontAwesomeIcon,
  NxForm,
  NxFormGroup,
  NxFormRow,
  NxFormSelect,
  NxH1,
  NxH2,
  NxInfoAlert,
  NxPageTitle,
  NxStatefulForm,
  NxTextInput,
  NxTile
} from '@globalart/react-shared-components';
import { faPlus, faUpload, faTrashAlt } from '@fortawesome/free-solid-svg-icons';
import { filter, isEmpty, keys, map, match, mapObjIndexed, path, split, test, values } from 'ramda';

import { FormUtils, Page } from '@globalart/astrum-ui-plugin';

import UploadStrings from '../../../../constants/pages/browse/upload/UploadStrings';

import machine from './UploadDetailsMachine.js';
import { COMPOUND_FIELD_PARENT_NAME, ASSET_NUM_MATCHER, MAVEN_FORMAT, MAVEN_COMPONENT_COORDS_GROUP }
  from './UploadDetailsUtils.js';

import './UploadDetails.scss';
import { useCurrentStateAndParams, useRouter } from '@uirouter/react';
import { ROUTE_NAMES } from '../../../../routerConfig/routeNames/routeNames.js';

/**
 * React component that renders a form group from a componentField/assetField structure and the
 * machine state
 */
function Field({ displayName, helpText, name, type, optional, options, machineState, send, validatable }) {
  const disabled = !!path(split('.', name), machineState.context.disabledFields);

  // Handle BOOLEAN type
  if (type === 'BOOLEAN') {
    return (
      <NxCheckbox { ...FormUtils.checkboxProps(name, machineState) }
                  disabled={disabled}
                  onChange={FormUtils.handleUpdate(name, send)}>
        {displayName}
      </NxCheckbox>
    );
  }

  // Handle SELECT type
  if (type === 'SELECT') {
    return (
      <NxFormGroup label={displayName} sublabel={helpText} isRequired={!optional}>
        <NxFormSelect { ...FormUtils.fieldProps(name, machineState) }
                      disabled={disabled}
                      onChange={FormUtils.handleUpdate(name, send)}
                      validatable={!optional || validatable}>
          <option value="">Select...</option>
          {options && Object.entries(options).map(([key, value]) => (
            <option key={key} value={key}>
              {value}
            </option>
          ))}
        </NxFormSelect>
      </NxFormGroup>
    );
  }

  // Default STRING type
  return (
    <NxFormGroup label={displayName} sublabel={helpText} isRequired={!optional}>
      <NxTextInput { ...FormUtils.fieldProps(name, machineState) }
                    disabled={disabled}
                   onChange={FormUtils.handleUpdate(name, send)}
                   validatable={!optional || validatable} />
    </NxFormGroup>
  );
}

export default function UploadDetails() {
  const { params } = useCurrentStateAndParams();
  const itemId = params?.itemId;
  const router = useRouter();
  const onCancel = useCallback(() => router.stateService.go(ROUTE_NAMES.BROWSE.UPLOAD.LIST));

  const [state, send] = useMachine(machine, {
        context: {
          repoId: decodeURIComponent(itemId)
        },
        devTools: true
      }),
      {
        multipleUpload,
        repoSettings,
        componentFieldsByGroup,
        assetFields,
        data,
        hasPomExtension,
        groupHelpText
      } = state.context,
      assetStateKeys = filter(test(ASSET_NUM_MATCHER), keys(data));

  const mkField = field => <Field key={field.name} { ...field } machineState={state} send={send} />,
      mkAssetField = prefix => field => mkField({
        ...field,
        name: `${prefix}.${field.name}`,
        validatable: true
      });

  function deleteAssetGroup(assetKey) {
    send({ type: 'DELETE_ASSET', assetKey });
  }

  return (
    <Page id="as-upload-details" data-testid="as-upload-details">
      <NxPageTitle>
        <NxH1 id="upload-details-title">
          <NxFontAwesomeIcon icon={faUpload} />
          <span>{UploadStrings.UPLOAD.DETAILS.TITLE}</span>
        </NxH1>
        <NxPageTitle.Description id="upload-details-description">
          {UploadStrings.UPLOAD.DETAILS.DESCRIPTION}
        </NxPageTitle.Description>
      </NxPageTitle>
      {/* Not using NxTile because it renders a <section> which we don't want in this case since this tile
        * contains multiple top-level sections
        */}
      <div className="nx-tile">
        <NxStatefulForm { ...FormUtils.formProps(state, send) }
                        onCancel={() => {
                          send({type: 'CANCEL'});
                          onCancel();
                        }}
                        submitBtnText={UploadStrings.UPLOAD.DETAILS.SUBMIT_BTN_LABEL}
                        aria-labelledby="upload-details-title"
                        aria-describedby="upload-details-description">
          {() =>
            <NxTile.Content>
              <NxForm.RequiredFieldNotice />
              <NxTile.Subsection aria-labelledby="upload-details-tile-title">
                <NxTile.SubsectionHeader>
                  <NxH2 id="upload-details-tile-title">
                    {UploadStrings.UPLOAD.DETAILS.TILE_TITLE(repoSettings?.name)}
                  </NxH2>
                </NxTile.SubsectionHeader>
                { map(assetKey => {
                    const fileUploadName = `${assetKey}.${COMPOUND_FIELD_PARENT_NAME}`,
                        assetNum = parseInt(match(ASSET_NUM_MATCHER, assetKey)[1]),
                        groupName = UploadStrings.UPLOAD.DETAILS.ASSET_GROUP_NAME(assetNum + 1);

                    return (
                      <fieldset aria-label={groupName} className="as-upload-details__asset-group" key={assetKey}>
                        <NxFormGroup label={UploadStrings.UPLOAD.DETAILS.FILE_UPLOAD_LABEL} isRequired>
                          <NxFileUpload isRequired
                                        { ...FormUtils.fileUploadProps(fileUploadName, state) }
                                        onChange={FormUtils.handleUpdate(fileUploadName, send)} />
                        </NxFormGroup>
                        { !(!assetFields || isEmpty(assetFields)) &&
                          <NxFormRow>
                            { map(mkAssetField(assetKey), assetFields) }
                            { assetStateKeys.length > 1 &&
                              <NxButtonBar>
                                <NxButton type="button" variant="tertiary" onClick={() => deleteAssetGroup(assetKey)}>
                                  <NxFontAwesomeIcon icon={faTrashAlt} />
                                  <span>Delete</span>
                                </NxButton>
                              </NxButtonBar>
                            }
                          </NxFormRow>
                        }
                      </fieldset>
                    );
                  }, assetStateKeys)
                }
                { multipleUpload &&
                  <NxFormRow className="as-upload-details__add-asset-row">
                    <NxButtonBar>
                      <NxButton type="button" variant="tertiary" onClick={() => send({type: 'ADD_ASSET'})}>
                        <NxFontAwesomeIcon icon={faPlus} />
                        <span>{UploadStrings.UPLOAD.DETAILS.ADD_ANOTHER_ASSET_BTN_LABEL}</span>
                      </NxButton>
                    </NxButtonBar>
                  </NxFormRow>
                }
              </NxTile.Subsection>
              { values(mapObjIndexed((fields, group) => {
                  const sectionId = `upload-details-group-${group.toLowerCase().replace(' ', '-')}`,
                      isMavenComponentCoords =
                          repoSettings.format === MAVEN_FORMAT && group === MAVEN_COMPONENT_COORDS_GROUP,
                      disabled = isMavenComponentCoords && hasPomExtension,
                      groupHelp = groupHelpText && groupHelpText[group];

                  return (
                    <NxTile.Subsection aria-labelledby={sectionId} key={group}>
                      <NxTile.SubsectionHeader>
                        <NxH2 id={sectionId}>{group}</NxH2>
                        {groupHelp && <p className="nx-sub-label">{groupHelp}</p>}
                      </NxTile.SubsectionHeader>
                      { disabled &&
                        <NxInfoAlert>{UploadStrings.UPLOAD.DETAILS.COORDINATES_EXTRACTED_FROM_POM_MESSAGE}</NxInfoAlert>
                      }
                      { map(mkField, fields) }
                    </NxTile.Subsection>
                  );
                }, componentFieldsByGroup))
              }
            </NxTile.Content>
          }
        </NxStatefulForm>
      </div>
    </Page>
  );
}
