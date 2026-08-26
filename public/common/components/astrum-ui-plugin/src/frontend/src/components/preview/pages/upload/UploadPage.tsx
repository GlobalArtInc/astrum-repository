

import React from 'react';
import { useCurrentStateAndParams } from '@uirouter/react';

import { UploadRepositoryListPage } from './UploadRepositoryListPage';
import { UploadFormContainer } from './UploadFormContainer';

/**
 * UploadPage is the main entry point for the Upload module.
 * It handles routing between the repository list and the upload form.
 *
 * @returns {JSX.Element} The rendered component.
 */
export function UploadPage(): JSX.Element {
  const { params } = useCurrentStateAndParams();
  const isFormView = Boolean(params?.repoName);

  return isFormView ? <UploadFormContainer /> : <UploadRepositoryListPage />;
}

export default UploadPage;
