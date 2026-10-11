

import React, { useCallback } from 'react';
import { useCurrentStateAndParams, useRouter } from '@uirouter/react';
import { Box, Callout, Flex, Spinner, Text } from '@radix-ui/themes';
import { AlertCircle } from 'lucide-react';

import { Breadcrumbs } from '../search/details/Breadcrumbs';

import { useUploadDefinition } from './hooks/useUploadDefinition';
import { useUploadableRepositories } from './hooks/useUploadableRepositories';
import { UploadForm } from './UploadForm';

import './UploadFormPage.scss';
import UploadStrings from '../../constants/pages/upload/UploadStrings';

/**
 * Page shell for the Upload Form.
 *
 * Reads route params, fetches the upload definition, and delegates
 * all form rendering to UploadForm once data is ready.
 */
export function UploadFormContainer(): JSX.Element {
  const { params } = useCurrentStateAndParams();
  const router = useRouter();
  const repositoryName = params?.repoName
    ? decodeURIComponent(params.repoName)
    : '';

  const {
    loading,
    error,
    repositorySettings,
    componentFields,
    componentFieldsByGroup,
    assetFields,
    multipleUpload,
    regexMap,
  } = useUploadDefinition(repositoryName);

  const { repositories: availableRepositories } = useUploadableRepositories();

  const handleBack = useCallback(() => {
    router.stateService.go('preview.browse.upload.list');
  }, [router]);

  const handleRepositoryChange = useCallback(
    (newRepoName: string) => {
      if (newRepoName && newRepoName !== repositoryName) {
        router.stateService.go('preview.browse.upload.form', { repoName: newRepoName });
      }
    },
    [router, repositoryName],
  );

  const breadcrumbItems = [
    { label: UploadStrings.FORM_CONTAINER.UPLOAD, onClick: handleBack },
    ...(repositoryName ? [{ label: repositoryName }] : []),
  ];

  if (loading) {
    return (
      <Box className="upload-form-page">
        <Breadcrumbs items={breadcrumbItems} />
        <Flex direction="column" align="center" justify="center" gap="2" className="upload-form-page__loading">
          <Box className="upload-form-page__loading-spinner">
            <Spinner size="3" />
          </Box>
          <Text size="3" weight="medium">{UploadStrings.FORM_CONTAINER.LOADING_UPLOAD_CONFIGURATION}</Text>
          <Text size="2" color="gray">{UploadStrings.FORM_CONTAINER.PREPARING_FORM(repositoryName)}</Text>
        </Flex>
      </Box>
    );
  }

  if (error) {
    return (
      <Box className="upload-form-page">
        <Breadcrumbs items={breadcrumbItems} />
        <Box className="upload-form-page__error">
          <Callout.Root color="red" size="2">
            <Callout.Icon>
              <AlertCircle size={16} />
            </Callout.Icon>
            <Callout.Text>{error}</Callout.Text>
          </Callout.Root>
        </Box>
      </Box>
    );
  }

  return (
    <Box className="upload-form-page">
      <Breadcrumbs items={breadcrumbItems} />
      <UploadForm
        repositoryName={repositoryName}
        repositorySettings={repositorySettings}
        componentFields={componentFields}
        componentFieldsByGroup={componentFieldsByGroup}
        assetFields={assetFields}
        multipleUpload={multipleUpload}
        regexMap={regexMap}
        availableRepositories={availableRepositories}
        onRepositoryChange={handleRepositoryChange}
        onBack={handleBack}
      />
    </Box>
  );
}

export default UploadFormContainer;
