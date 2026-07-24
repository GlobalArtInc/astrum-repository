
import React from 'react';

import {
  NxH2,
  NxCheckbox,
  NxFieldset,
  NxFormGroup,
  NxFormSelect,
  NxWarningAlert
} from '@globalart/react-shared-components';
import {FormUtils, ExtJS} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../../constants/UIStrings';

import ToggleableTextInput from './ToggleableTextInput/ToggleableTextInput';

const {CONNECTORS, COSIGN} = UIStrings.REPOSITORIES.EDITOR.OCI;

/**
 * OciConnectorSettings - connector configuration for OCI v2 registries.
 *
 * Unlike Docker this facet exposes ONLY the v2 connector surface: HTTP port,
 * HTTPS port, optional subdomain, path-based routing toggle, and force-basic-auth.
 * No Docker v1 toggle is emitted; OCI repositories always speak the v2 distribution spec.
 *
 * Cosign keyless policy lives below the connector fields. The KEYLESS enforcement
 * option is intentionally hidden until a real Cosign verifier ships - the stub
 * verifier fails OPEN at both upload and pull time, so exposing the option would
 * give admins a false sense of supply-chain enforcement (UX P0-1 / ASTRUM-53064).
 */
export default function OciConnectorSettings({parentMachine}) {
  const [parentState, sendParent] = parentMachine;

  const repositoryName = parentState.context.data.name;
  const isProEdition = ExtJS.isProEdition();

  return (
    <>
      <NxH2 className="as-oci-connectors-caption">{CONNECTORS.CAPTION}</NxH2>

      <p className="as-oci-connectors-help">{CONNECTORS.HELP}</p>

      <NxFieldset
        label={CONNECTORS.PATH_ENABLED.LABEL}
        className="as-form-group-oci-connector-path-enabled"
      >
        <NxCheckbox
          {...FormUtils.checkboxProps('oci.pathEnabled', parentState)}
          onChange={FormUtils.handleUpdate('oci.pathEnabled', sendParent)}
        >
          {CONNECTORS.PATH_ENABLED.DESCR}
        </NxCheckbox>
      </NxFieldset>

      {isProEdition && (
        <ToggleableTextInput
          parentMachine={parentMachine}
          contextPropName="oci.subdomain"
          label={CONNECTORS.SUBDOMAIN.LABEL}
          sublabel={CONNECTORS.SUBDOMAIN.SUBLABEL}
          defaultValue={repositoryName}
          placeholder={CONNECTORS.SUBDOMAIN.PLACEHOLDER}
          clearIfDisabled
          className="as-form-group-oci-connector-subdomain"
        />
      )}

      <ToggleableTextInput
        parentMachine={parentMachine}
        contextPropName="oci.httpPort"
        label={CONNECTORS.HTTP.LABEL}
        sublabel={CONNECTORS.HTTP.SUBLABEL}
        placeholder={CONNECTORS.HTTP.PLACEHOLDER}
        clearIfDisabled
        className="as-form-group-oci-connector-http-port"
      />
      <ToggleableTextInput
        parentMachine={parentMachine}
        contextPropName="oci.httpsPort"
        label={CONNECTORS.HTTPS.LABEL}
        sublabel={CONNECTORS.HTTPS.SUBLABEL}
        placeholder={CONNECTORS.HTTPS.PLACEHOLDER}
        clearIfDisabled
        className="as-form-group-oci-connector-https-port"
      />

      <NxFieldset
        label={CONNECTORS.FORCE_BASIC_AUTH.LABEL}
        className="as-form-group-oci-force-basic-auth"
      >
        <NxCheckbox
          {...FormUtils.checkboxProps('oci.forceBasicAuth', parentState)}
          onChange={FormUtils.handleUpdate('oci.forceBasicAuth', sendParent)}
        >
          {CONNECTORS.FORCE_BASIC_AUTH.DESCR}
        </NxCheckbox>
        {/* ASTRUM-53064 B2: surface the security implication of disabling
            Force Basic Authentication as a warning banner so the admin cannot
            silently leave the registry open to anonymous pulls. */}
        <NxWarningAlert className="as-oci-force-basic-auth-warning">
          {CONNECTORS.FORCE_BASIC_AUTH.WARNING}
        </NxWarningAlert>
      </NxFieldset>

      <NxH2 className="as-oci-cosign-caption">{COSIGN.CAPTION}</NxH2>
      <p className="as-oci-cosign-help">{COSIGN.HELP}</p>

      <NxFormGroup
        label={COSIGN.ENFORCEMENT.LABEL}
        sublabel={COSIGN.ENFORCEMENT.SUBLABEL}
        className="as-form-group-oci-cosign-enforcement"
      >
        <NxFormSelect
          {...FormUtils.selectProps('oci.cosign.enforcement', parentState)}
          onChange={FormUtils.handleUpdate('oci.cosign.enforcement', sendParent)}
        >
          {/*
            UX P0-1 / ASTRUM-53064: KEYLESS is intentionally NOT exposed in the
            dropdown until a real Cosign verifier ships. The current
            OciCosignKeylessVerifierNoop fails open at both upload AND pull
            time, so showing this option would let admins believe they had
            configured signature enforcement when in reality every signature
            is accepted. Re-enable when a non-stub verifier is wired in.
          */}
          <option value="NONE">{COSIGN.ENFORCEMENT.OPTIONS.NONE}</option>
        </NxFormSelect>
      </NxFormGroup>
    </>
  );
}
