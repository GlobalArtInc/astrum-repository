
import React from 'react';
import {useActor} from '@xstate/react';
import classNames from 'classnames';
import {
  AsCard,
  AsFontAwesomeIcon,
  AsH3,
  AsP,
  AsTextLink,
  AsTooltip,
  AsLoadingSpinner,
  AsButton,
} from '@globalart/react-shared-components';

import {ExtJS, DateUtils} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../../constants/UIStrings';
import {
  faSync,
  faCheckCircle,
  faTimesCircle,
} from '@fortawesome/free-solid-svg-icons';

import './SupportZipHa.scss';

const {SUPPORT_ZIP: LABELS} = UIStrings;

export default function NodeCard({actor, createZip, isBlobStoreConfigured}) {
  const [state] = useActor(actor);

  const {data} = state.context;
  const isNodeActive = data.status !== 'NODE_UNAVAILABLE';
  const zipNotCreated = data.status === 'NOT_CREATED';
  const zipCreating = data.status === 'CREATING';
  const nodeError = data.status === 'FAILED';

  const zipLastUpdatedHtml = () => {
    if (!isNodeActive) {
      return (
        <AsP className="as-p-zip-updated">
          {LABELS.NODE_UNAVAILABLE_CANNOT_CREATE}
        </AsP>
      );
    }

    if (!isBlobStoreConfigured) {
      return (
        <AsP className="as-p-zip-updated">
          {LABELS.NO_BLOB_STORE_CONFIGURED}
        </AsP>
      );
    }

    if (zipCreating) {
      return <AsLoadingSpinner>{LABELS.CREATING_ZIP}</AsLoadingSpinner>;
    }

    if (zipNotCreated) {
      return <AsP className="as-p-zip-updated">{LABELS.NO_ZIP_CREATED}</AsP>;
    }

    return (
      <AsP className="as-p-zip-updated">
        <AsTextLink href={ExtJS.urlOf(`service/rest/wonderland/download/${data.blobRef}`)} download>
          {LABELS.DOWNLOAD_ZIP}
          <br />
          {DateUtils.prettyDateTime(new Date(data.lastUpdated))}
        </AsTextLink>
      </AsP>
    );
  };

  const generateButtonDisabled =
    !isNodeActive || !isBlobStoreConfigured || zipCreating;

  const handleGenerate = () => {
    if (!generateButtonDisabled) {
      createZip();
    }
  };

  return (
    <AsCard>
      <AsTooltip
        title={isNodeActive ? LABELS.NODE_IS_ACTIVE : LABELS.NODE_IS_INACTIVE}
        placement="top-middle"
      >
        <AsCard.Header>
          <AsH3>
            {isNodeActive ? (
              <AsFontAwesomeIcon
                icon={faCheckCircle}
                className="as-node-green-checkmark"
              />
            ) : (
              <AsFontAwesomeIcon icon={faTimesCircle} />
            )}{' '}
            {data.hostname}
          </AsH3>
        </AsCard.Header>
      </AsTooltip>
      <AsCard.Content>
        <AsCard.Text>{zipLastUpdatedHtml()}</AsCard.Text>
      </AsCard.Content>
      <AsCard.Footer>
        {nodeError && (
          <AsP className="error-message">{LABELS.GENERATE_ERROR}</AsP>
        )}
        <AsButton
          variant={nodeError ? 'error' : 'secondary'}
          onClick={handleGenerate}
          className={classNames({
            disabled: generateButtonDisabled,
          })}
        >
          {nodeError ? (
            <>
              <AsFontAwesomeIcon icon={faSync} />
              <span>{LABELS.RETRY}</span>
            </>
          ) : (
            LABELS.GENERATE_NEW_ZIP_FILE
          )}
        </AsButton>
      </AsCard.Footer>
    </AsCard>
  );
}
