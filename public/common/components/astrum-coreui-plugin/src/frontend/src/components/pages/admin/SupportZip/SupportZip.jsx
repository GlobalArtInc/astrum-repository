
import React from 'react';

import SupportZipHa from "./HA/SupportZipHa";
import SupportZipSingleNode from "./SupportZipSingleNode";

import {ExtJS} from "@globalart/astrum-ui-plugin";

export default function SupportZip() {
  const isClustered = ExtJS.state().getValue('astrum.datastore.clustered.enabled');

  return <>
    {isClustered && <SupportZipHa />}
    {!isClustered && <SupportZipSingleNode /> }
  </>;
}
