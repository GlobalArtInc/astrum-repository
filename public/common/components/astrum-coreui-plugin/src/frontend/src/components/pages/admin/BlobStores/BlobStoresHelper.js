

import {APIConstants} from '@globalart/astrum-ui-plugin';

const {
  REST: {
    PUBLIC: {BLOB_STORES, BLOB_STORES_CONVERT_TO_GROUP},
    INTERNAL: {BLOB_STORES_TYPES, BLOB_STORES_QUOTA_TYPES, BLOB_STORES_USAGE},
  },
} = APIConstants;

const deleteBlobStoreUrl = (name) => `${BLOB_STORES}/${encodeURIComponent(name)}`;
const convertToGroupBlobStoreUrl = (name, newName) =>
    `${BLOB_STORES_CONVERT_TO_GROUP}/${encodeURIComponent(name)}/${encodeURIComponent(newName)}`;
const createBlobStoreUrl = (typeId) => `${BLOB_STORES}/${encodeURIComponent(typeId)}`;
const singleBlobStoreUrl = (typeId, name) => `${BLOB_STORES}/${encodeURIComponent(typeId)}/${encodeURIComponent(name)}`;
const blobStoreTypesUrl = `/${BLOB_STORES_TYPES}`;
const blobStoreQuotaTypesUrl = `/${BLOB_STORES_QUOTA_TYPES}`;
const blobStoreUsageUrl = (name) => `/${BLOB_STORES_USAGE}/${encodeURIComponent(name)}`;

export const URLs = {
  deleteBlobStoreUrl,
  convertToGroupBlobStoreUrl,
  createBlobStoreUrl,
  singleBlobStoreUrl,
  blobStoreTypesUrl,
  blobStoreQuotaTypesUrl,
  blobStoreUsageUrl,
};
