
import React from 'react';

import {Information} from '@globalart/astrum-ui-plugin';
import {Section} from '@globalart/astrum-ui-plugin';

/**
 * @since 3.24
 * @param sectionName - the name of the nested section to display
 * @param sectionInformation - system information that has objects as the value instead of simple strings
 */
export default function NestedSystemInformationSection({sectionName, sectionInformation}) {
  return <Section>
    <h2>{sectionName}</h2>
    {Object.entries(sectionInformation).map(([nestedName, nestedInformation]) =>
        <div key={nestedName}>
          <h3>{nestedName}</h3>
          <Information information={nestedInformation}/>
        </div>
    )}
  </Section>;
}
