
import React from 'react';
import {AsWarningAlert} from '@globalart/astrum-react-shared-components';

import UIStrings from '../../../../../constants/UIStrings'

import './S3BlobStoreWarning.scss';

const FIELDS = UIStrings.S3_BLOBSTORE_CONFIGURATION;

export default function S3BlobStoreWarning() {

  return <AsWarningAlert><span dangerouslySetInnerHTML={{__html:FIELDS.S3BlobStore_Help}}/></AsWarningAlert>;
}
