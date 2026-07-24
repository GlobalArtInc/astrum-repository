

import S3BlobStoreSettings from './S3/S3BlobStoreSettings';
import S3BlobStoreWarning from './S3/S3BlobStoreWarning';
import S3BlobStoreActions from './S3/S3BlobStoreActions';
import AzureBlobStoreSettings from './Azure/AzureBlobStoreSettings';
import AzureBlobStoreActions from './Azure/AzureBlobStoreActions';
import GoogleBlobStoreSettings from './Google/GoogleBlobStoreSettings';
import GoogleBlobStoreActions from './Google/GoogleBlobStoreActions';
import FileBlobStoreWarning from './File/FileBlobStoreWarning';

const BlobStoreTypes = {
  azure: {
    Settings: AzureBlobStoreSettings,
    Actions: AzureBlobStoreActions
  },
  s3: {
    Settings: S3BlobStoreSettings,
    Warning: S3BlobStoreWarning,
    Actions: S3BlobStoreActions
  },
  google: {
    Settings: GoogleBlobStoreSettings,
    Actions: GoogleBlobStoreActions
  },
  file: {
    Warning: FileBlobStoreWarning
  }
};

export default BlobStoreTypes;
