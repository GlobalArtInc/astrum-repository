
import React from 'react';
import UIStrings from '../../../../constants/UIStrings';
import {FieldWrapper, Textfield} from '@globalart/astrum-ui-plugin';
import {NxButton, NxFontAwesomeIcon} from '@globalart/react-shared-components';
import {faDownload} from '@fortawesome/free-solid-svg-icons';

import {SectionFooter} from '@globalart/astrum-ui-plugin';

export default function NodeSupportZipResponse({response, download}) {
  return <div className='as-support-zip-response-ha'>
    <h2>{UIStrings.SUPPORT_ZIP.CREATED_TITLE}</h2>
    <p dangerouslySetInnerHTML={{__html: UIStrings.SUPPORT_ZIP.CREATED_DESCRIPTION}}></p>
    <FieldWrapper labelText={UIStrings.SUPPORT_ZIP.CREATED_NODEID_LABEL}>
      <Textfield
        name='nodeId'
        disabled
        value={response.nodeId}
      />
    </FieldWrapper>
    <FieldWrapper labelText={UIStrings.SUPPORT_ZIP.CREATED_NAME_LABEL}>
      <Textfield
        name='name'
        disabled
        value={response.name}
      />
    </FieldWrapper>
    <FieldWrapper labelText={UIStrings.SUPPORT_ZIP.CREATED_SIZE_LABEL}>
      <Textfield
        name='size'
        disabled
        value={response.size}
      />
    </FieldWrapper>
    <FieldWrapper labelText={UIStrings.SUPPORT_ZIP.CREATED_PATH_LABEL}>
      <Textfield
        name='file'
        disabled
        value={response.file}
      />
    </FieldWrapper>
    <SectionFooter>
      <NxButton variant="primary" onClick={(event) => download(event, response.name)} type="submit">
        <NxFontAwesomeIcon icon={faDownload}/>
        <span>{UIStrings.SUPPORT_ZIP.CREATED_DOWNLOAD_BUTTON}</span>
      </NxButton>
    </SectionFooter>
  </div>;
}
