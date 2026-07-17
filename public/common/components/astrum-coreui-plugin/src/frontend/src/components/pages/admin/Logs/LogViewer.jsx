
import React, {useEffect, useRef} from 'react';
import {useMachine} from '@xstate/react';

import {
  ContentBody,
  ExtJS,
  PageActions,
  PageHeader,
  PageTitle,
  Section,
  SectionToolbar
} from '@globalart/astrum-ui-plugin';
import {
  NxButton,
  NxButtonBar,
  NxFontAwesomeIcon,
  NxFormGroup,
  NxFormSelect,
  NxLoadWrapper,
  NxTextInput,
} from '@globalart/react-shared-components';

import {faStamp} from '@fortawesome/free-solid-svg-icons';

import LogViewerMachine from "./LogViewerMachine";
import UIStrings from '../../../../constants/UIStrings';

const VIEW = UIStrings.LOGS.VIEW;

export default function LogViewer({itemId}) {
  const refreshPeriods = [
    [0,   UIStrings.LOGS.REFRESH.MANUAL_ITEM],
    [20,  UIStrings.LOGS.REFRESH.TWENTY_SECONDS_ITEM],
    [60,  UIStrings.LOGS.REFRESH.MINUTE_ITEM],
    [120, UIStrings.LOGS.REFRESH.TWO_MINUTES_ITEM],
    [300, UIStrings.LOGS.REFRESH.FIVE_MINUTES_ITEM]
  ];
  const refreshSizes = [
    [25,  UIStrings.LOGS.SIZE.LAST25KB_ITEM],
    [50,  UIStrings.LOGS.SIZE.LAST50KB_ITEM],
    [100, UIStrings.LOGS.SIZE.LAST100KB_ITEM]
  ];
  const [current, send] = useMachine(LogViewerMachine, {
    context: {
      itemId
    },
    devTools: true
  });
  const selectedPeriod = current.context.period;
  const selectedSize = current.context.size;
  const mark = current.context.mark;
  const logText = current.context.data;
  const loadError = current.matches('error') ? current.context.error : null;
  const textarea = useRef(null);

  function retry() {
    send({type: 'RETRY'});
  }

  // scroll to bottom of text area only when log content changes
  useEffect(() => {
    if (textarea.current) {
      textarea.current.scrollTop = textarea.current.scrollHeight;
    }
  }, [logText]);

  function onChangePeriod(period) {
    const newPeriod = Number(period);
    if (newPeriod === 0) {
      send({type: 'MANUAL_REFRESH', period: newPeriod});
    }
    else {
      send({type: 'UPDATE_PERIOD', period: newPeriod});
    }
  }

  function onChangeSize(size) {
    send({type: 'UPDATE_SIZE', size});
  }

  function updateMark(value) {
    send({type: 'UPDATE_MARK', mark: value});
  }

  function onMarkKeyPress(event) {
    if (event.key === 'Enter') {
      event.preventDefault();
      send({type: 'INSERT_MARK'});
    }
  }

  function insertMark() {
    send({type: 'INSERT_MARK'});
  }

  return <div className="as-logs">
    <PageHeader>
      <PageTitle text={VIEW.TITLE(decodeURIComponent(itemId))}/>
      <PageActions>
        <a
            download
            className="nx-btn nx-btn--primary"
            href={ExtJS.urlOf(`service/rest/internal/logging/logs/${itemId}`)}
            data-analytics-id="as-logs-download"
        >
          {VIEW.DOWNLOAD}
        </a>
      </PageActions>
    </PageHeader>
    <ContentBody>
      <Section className="as-log-viewer">
        <SectionToolbar>
          {itemId === "astrum.log" && <div className="nx-form-row">
            <NxFormGroup label={VIEW.MARK.LABEL}>
              <NxTextInput
                  className="as-log-viewer-mark-text"
                  id="mark"
                  name="mark"
                  value={mark}
                  isPristine={true}
                  onChange={updateMark}
                  onKeyPress={onMarkKeyPress}
              />
            </NxFormGroup>
            <NxButtonBar>
              <NxButton onClick={insertMark} id="insertMark" data-analytics-id="as-logs-mark">
                <NxFontAwesomeIcon icon={faStamp}/>
                <span>{VIEW.MARK.INSERT}</span>
              </NxButton>
            </NxButtonBar>
          </div>}

          <div className="as-spacer" />

          <div className="nx-form-row">
            <NxFormGroup label={VIEW.REFRESH.RATE_LABEL}>
              <NxFormSelect name="period" id="period" value={selectedPeriod} onChange={onChangePeriod}>
                {refreshPeriods.map(([period, label]) =>
                    <option key={period} value={period}>{label}</option>
                )}
              </NxFormSelect>
            </NxFormGroup>
            <NxFormGroup className="log-viewer-size" label={VIEW.REFRESH.SIZE_LABEL}>
              <NxFormSelect name="size" value={selectedSize} onChange={onChangeSize}>
                {refreshSizes.map(([size, label]) =>
                    <option key={size} value={size}>{label}</option>
                )}
              </NxFormSelect>
            </NxFormGroup>
          </div>
        </SectionToolbar>
        {/* loading={false} intentionally: existing content stays visible during background auto-refresh */}
        <NxLoadWrapper loading={false} error={loadError} retryHandler={retry}>
          <textarea
              name="logs"
              value={logText}
              className="log-viewer-textarea"
              ref={textarea}
              readOnly
          />
        </NxLoadWrapper>
      </Section>
    </ContentBody>
  </div>;
}
