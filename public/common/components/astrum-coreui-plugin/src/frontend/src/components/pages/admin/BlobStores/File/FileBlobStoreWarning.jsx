
import React from 'react';
import {useActor} from '@xstate/react';
import {NxWarningAlert} from '@globalart/react-shared-components';
import {ExtJS} from '@globalart/astrum-ui-plugin';
import UIStrings from '../../../../../constants/UIStrings';

const {BLOB_STORES: {FORM: {FILE_WARNING}}} = UIStrings;

function isRelativePath(path) {
  if (!path) {
    return false;
  }
  return !path.startsWith('/') && !path.match(/^[A-Za-z]:\\/);
}

function isUnderWorkDirectory(path, workDirectory) {
  if (!path || !workDirectory) {
    return false;
  }
  const normalizedPath = path.replace(/\\/g, '/');
  let normalizedWorkDir = workDirectory.replace(/\\/g, '/');

  // Ensure work directory ends with separator to avoid false positives
  // e.g., "/astrum-data/info" vs "/astrum-data/information"
  if (!normalizedWorkDir.endsWith('/')) {
    normalizedWorkDir += '/';
  }

  return normalizedPath.startsWith(normalizedWorkDir);
}

export default function FileBlobStoreWarning({service}) {
  const [state] = useActor(service);
  const {data} = state.context;
  const path = data?.path;

  const clusteringState = ExtJS.state()?.getValue('clustering');
  const isClustered = clusteringState?.isClustered || false;
  const workDirectory = clusteringState?.workDirectory || '';

  if (!isClustered) {
    return null;
  }

  const showWarning = isRelativePath(path) || isUnderWorkDirectory(path, workDirectory);

  if (!showWarning) {
    return null;
  }

  return (
    <NxWarningAlert>
      <strong>{FILE_WARNING.TITLE}</strong>
      <br />
      {FILE_WARNING.MESSAGE}
    </NxWarningAlert>
  );
}
