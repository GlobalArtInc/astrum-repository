
import React from 'react';

import BlobStoreTypes from './BlobStoreTypes';

export default function BlobStoreWarning({type, service}) {
  const Warning = BlobStoreTypes[type.id]?.Warning;

  if (Warning) {
    return <Warning service={service} />;
  }

  return null;
}
