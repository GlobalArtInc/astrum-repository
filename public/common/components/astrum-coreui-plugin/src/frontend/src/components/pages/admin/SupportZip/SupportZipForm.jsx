
import React, { FormEvent } from 'react';
import UIStrings from '../../../../constants/UIStrings';
import {FormUtils} from '@globalart/astrum-ui-plugin';
import {AsButton, AsCheckbox, AsFieldset, AsFontAwesomeIcon, AsModal, useToggle, AsFormSelect, asFormSelectStateHelpers, AsFormGroup} from '@globalart/astrum-react-shared-components';
import {faFileArchive} from '@fortawesome/free-solid-svg-icons';

export default function SupportZipForm({params, setParams, submit, clustered, hazips, cancel}) {

  return <>
    <AsModal.Content>
      <div className="as-form">
        {cancel
        && <div className='as-support-zip-modal-form-vertical-spacer'/>
        }

        <div dangerouslySetInnerHTML={{__html: UIStrings.SUPPORT_ZIP.DESCRIPTION}}/>
        <AsFieldset label={UIStrings.SUPPORT_ZIP.CONTENTS}>
          <AsCheckbox
              checkboxId='systemInformation'
              isChecked={params.systemInformation}
              onChange={(value) => setParams('systemInformation', value)}
          >
            {UIStrings.SUPPORT_ZIP.REPORT_LABEL}
          </AsCheckbox>
          <AsCheckbox
              checkboxId='threadDump'
              isChecked={params.threadDump}
              onChange={(value) => setParams('threadDump', value)}
          >
            {UIStrings.SUPPORT_ZIP.DUMP_LABEL}
          </AsCheckbox>
          <AsCheckbox
              checkboxId='configuration'
              isChecked={params.configuration}
              onChange={(value) => setParams('configuration', value)}
          >
            {UIStrings.SUPPORT_ZIP.CONFIGURATION_LABEL}
          </AsCheckbox>
          <AsCheckbox
              checkboxId='security'
              isChecked={params.security}
              onChange={(value) => setParams('security', value)}
          >
            {UIStrings.SUPPORT_ZIP.SECURITY_LABEL}
          </AsCheckbox>
          <AsCheckbox
              checkboxId='log'
              isChecked={params.log}
              onChange={(value) => setParams('log', value)}
          >
            {UIStrings.SUPPORT_ZIP.LOGFILES_LABEL}
          </AsCheckbox>
          <AsCheckbox
              checkboxId='taskLog'
              isChecked={params.taskLog}
              onChange={(value) => setParams('taskLog', value)}
          >
            {UIStrings.SUPPORT_ZIP.TASKLOGFILES_LABEL}
          </AsCheckbox>
          <AsCheckbox
              checkboxId='replication'
              isChecked={params.replication}
              onChange={(value) => setParams('replication', value)}
          >
            {UIStrings.SUPPORT_ZIP.REPLICATION_LABEL}
          </AsCheckbox>
          <AsCheckbox
              checkboxId='auditLog'
              isChecked={params.auditLog}
              onChange={(value) => setParams('auditLog', value)}
          >
            {UIStrings.SUPPORT_ZIP.AUDITLOGFILES_LABEL}
          </AsCheckbox>
          <AsCheckbox
              checkboxId='metrics'
              isChecked={params.metrics}
              onChange={(value) => setParams('metrics', value)}
          >
            {UIStrings.SUPPORT_ZIP.METRICS_LABEL}
          </AsCheckbox>
          <AsCheckbox
              checkboxId='jmx'
              isChecked={params.jmx}
              onChange={(value) => setParams('jmx', value)}
          >
            {UIStrings.SUPPORT_ZIP.JMX_LABEL}
          </AsCheckbox>
          <AsFormGroup label={UIStrings.SUPPORT_ZIP.ARCHIVED_LOGS_LABEL}>
                <AsFormSelect id='archivedLog' onChange={ (value) => setParams('archivedLog', value)}>
                  <option value="0">{UIStrings.PAGES.SUPPORT_ZIP.LIMIT_NONE}</option>
                  <option value="1">{UIStrings.PAGES.SUPPORT_ZIP.LIMIT_1_DAY}</option>
                  <option value="2">{UIStrings.PAGES.SUPPORT_ZIP.LIMIT_2_DAYS}</option>
                  <option value="3">{UIStrings.PAGES.SUPPORT_ZIP.LIMIT_3_DAYS}</option>
                </AsFormSelect>
          </AsFormGroup>
        </AsFieldset>
        <AsFieldset label={UIStrings.SUPPORT_ZIP.OPTIONS}>
          <AsCheckbox
              checkboxId='limitFileSizes'
              isChecked={params.limitFileSizes}
              onChange={(value) => setParams('limitFileSizes', value)}
          >
            {UIStrings.SUPPORT_ZIP.LIMITFILESIZES_LABEL}
          </AsCheckbox>
          <AsCheckbox
              checkboxId='limitZipSize'
              isChecked={params.limitZipSize}
              onChange={(value) => setParams('limitZipSize', value)}
          >
            {UIStrings.SUPPORT_ZIP.LIMITZIPSIZE_LABEL}
          </AsCheckbox>
        </AsFieldset>
      </div>
    </AsModal.Content>

    <footer className="as-footer">
      <div className="as-btn-bar">
        {cancel &&
        <AsButton variant='secondary' onClick={cancel}>
          <span>{UIStrings.SETTINGS.CANCEL_BUTTON_LABEL}</span>
        </AsButton>
        }

        <AsButton variant='primary' onClick={submit} type='submit'>
          <AsFontAwesomeIcon icon={faFileArchive}/>
          <span>{UIStrings.PAGES.SUPPORT_ZIP.CREATE}</span>
        </AsButton>
        {clustered &&
        <AsButton variant='primary' onClick={hazips} type='submit'>
          <AsFontAwesomeIcon icon={faFileArchive}/>
          <span>{UIStrings.PAGES.SUPPORT_ZIP.CREATE_ALL_NODES}</span>
        </AsButton>
        }
      </div>
    </footer>
  </>;
}
