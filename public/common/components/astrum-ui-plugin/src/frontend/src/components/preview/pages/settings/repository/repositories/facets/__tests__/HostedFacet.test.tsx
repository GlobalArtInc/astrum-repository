

import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Theme } from '@radix-ui/themes';

import { HostedFacet } from '../../facets/HostedFacet';

const DEPLOYMENT_POLICY_TESTID = 'select-storage-writePolicy';
const PROPRIETARY_COMPONENTS_TESTID = 'checkbox-component-proprietaryComponents';
const HOSTED_HEADING = 'Hosted';

const defaultFormData = {
  name: 'test-repo',
  type: 'hosted' as const,
  format: 'maven2',
  online: true,
  storage: {
    blobStoreName: 'default',
    strictContentTypeValidation: true,
    writePolicy: 'ALLOW_ONCE' as const,
  },
  component: {
    proprietaryComponents: false,
  },
};

function renderHostedFacet(props: Partial<Parameters<typeof HostedFacet>[0]> = {}) {
  const defaultProps = {
    formData: defaultFormData,
    onChange: jest.fn(),
    onNestedChange: jest.fn(),
    errors: undefined,
  };

  return render(
    <Theme>
      <HostedFacet {...defaultProps} {...props} />
    </Theme>
  );
}

describe('HostedFacet', () => {
  describe('deployment policy', () => {
    it('shouldRenderDeploymentPolicySelect', () => {
      renderHostedFacet();

      expect(screen.getByTestId(DEPLOYMENT_POLICY_TESTID)).toBeInTheDocument();
    });

    it('shouldRenderHostedSectionHeading', () => {
      renderHostedFacet();

      expect(screen.getByText(HOSTED_HEADING)).toBeInTheDocument();
    });
  });

  describe('proprietary components checkbox', () => {
    it('shouldRenderProprietaryComponentsForNonDockerFormats', () => {
      renderHostedFacet({ formData: { ...defaultFormData, format: 'maven2' } });

      expect(screen.getByTestId(PROPRIETARY_COMPONENTS_TESTID)).toBeInTheDocument();
    });

    it('shouldRenderProprietaryComponentsForNpmFormat', () => {
      renderHostedFacet({ formData: { ...defaultFormData, format: 'npm' } });

      expect(screen.getByTestId(PROPRIETARY_COMPONENTS_TESTID)).toBeInTheDocument();
    });

    it('shouldHideProprietaryComponentsForDockerFormat', () => {
      renderHostedFacet({ formData: { ...defaultFormData, format: 'docker' } });

      expect(screen.queryByTestId(PROPRIETARY_COMPONENTS_TESTID)).not.toBeInTheDocument();
    });
  });
});
