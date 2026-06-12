
import { faUpload } from '@fortawesome/free-solid-svg-icons';

export default {
  UPLOAD: {
    MENU: {
      text: 'Upload',
      description: 'Upload content to the hosted repository',
      icon: faUpload
    },
    EMPTY_MESSAGE: 'No repositories found.',
    LIST: {
      COLUMNS: {
        NAME: 'Name',
        FORMAT: 'Format',
        URL: 'URL',
      },
      COPY_URL_TITLE: 'Copy URL to Clipboard',
      FILTER_PLACEHOLDER: 'Filter'
    },
    DETAILS: {
      TITLE: 'Upload',
      DESCRIPTION: 'Upload content to the hosted repository',
      TILE_TITLE: repoName => `Choose Assets/Components for ${repoName} Repository`,
      FILE_UPLOAD_LABEL: 'File',
      SUBMIT_BTN_LABEL: 'Upload',
      ADD_ANOTHER_ASSET_BTN_LABEL: 'Add another asset',
      ASSET_GROUP_NAME: assetNum => `Asset ${assetNum}`,
      ASSET_NOT_UNIQUE_MESSAGE: 'Asset not unique',
      COORDINATES_EXTRACTED_FROM_POM_MESSAGE: 'Component details will be extracted from the provided POM file.'
    },
    URL_COPIED_MESSAGE: 'URL Copied to Clipboard',
    URL_COPY_ERROR_MESSAGE: 'Failed to copy URL to clipboard'
  }
};
