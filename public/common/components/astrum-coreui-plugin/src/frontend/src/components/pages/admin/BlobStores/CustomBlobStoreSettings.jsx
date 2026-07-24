
import React from 'react';

import BlobStoreTypes from './BlobStoreTypes';

export default function CustomBlobStoreSettings({type, service}) {
  const Settings = BlobStoreTypes[type.id]?.Settings;

  if (Settings) {
    return <Settings service={service} />;
  }

  return null;
}
