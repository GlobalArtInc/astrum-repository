
import React from 'react';

import {Detail, Master, MasterDetail} from '@globalart/astrum-ui-plugin';

import TagsList from './TagsList';
import TagsDetails from './TagsDetails';

export default function Tags() {
  return <MasterDetail path="browse/tags">
    <Master>
      <TagsList/>
    </Master>
    <Detail>
      <TagsDetails/>
    </Detail>
  </MasterDetail>;
}
