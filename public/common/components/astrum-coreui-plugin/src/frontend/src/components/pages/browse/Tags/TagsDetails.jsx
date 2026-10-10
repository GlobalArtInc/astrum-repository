
import React from 'react';
import {useMachine} from '@xstate/react';

import {ReadOnlyField} from '@globalart/astrum-ui-plugin';
import {
  AsLoadWrapper,
  AsTile,
  AsH2,
  AsTextLink,
  AsBackButton,
  AsReadOnly,
  AsCopyToClipboard
} from '@globalart/react-shared-components';

import {
  ContentBody,
  PageTitle,
} from '@globalart/astrum-ui-plugin';

import TagsDetailsMachine from './TagsDetailsMachine';
import { faTags } from '@fortawesome/free-solid-svg-icons';
import UIStrings from '../../../../constants/UIStrings';

const {TAGS} = UIStrings;

export default function TagsDetails({itemId}) {
  const [state, send] = useMachine(TagsDetailsMachine, {
    context: {
      pristineData: {
        name: itemId
      }
    },
    devTools: true
  });
  const isLoading = state.matches('loading');
  const tagData = state.context.data;
  const hasLoadError = state.matches('loadError') ? state.context.error : null;

  function retry() {
    send({type: 'RETRY'});
  }

  return <div className="as-tags">
    <div>
      <PageTitle icon={faTags} {...TAGS.DETAILS.HEADER}/>
      <AsBackButton href="#browse/tags" text={TAGS.DETAILS.BACK_TO_TAGS_TABLE}/>
    </div>
    <ContentBody>
      <AsTile>
        <AsLoadWrapper retryHandler={retry} loading={isLoading} error={hasLoadError}>
          {() => (
            <>
              <AsTile.Header>
                <AsTile.HeaderTitle>
                  <AsH2>{tagData.name} {TAGS.DETAILS.TILE_HEADER}</AsH2>
                </AsTile.HeaderTitle>
                <AsTile.HeaderActions>
                  <AsTextLink href={'#browse/search/custom/=' + encodeURIComponent(`tags="${tagData.name}"`)}>{TAGS.DETAILS.FIND_TAGGED}</AsTextLink>
                </AsTile.HeaderActions>
              </AsTile.Header>
              <AsTile.Content>
                <AsReadOnly>
                  <ReadOnlyField label={TAGS.DETAILS.FIRST_CREATED} value={new Date(tagData.firstCreated).toLocaleString()}/>
                  <ReadOnlyField label={TAGS.DETAILS.LAST_UPDATED} value={new Date(tagData.lastUpdated).toLocaleString()}/>
                </AsReadOnly>
                <AsCopyToClipboard 
                  label={TAGS.DETAILS.ATTRIBUTES}
                  content={JSON.stringify(tagData.attributes, null, 2)} 
                />
              </AsTile.Content>
            </>
          )}
        </AsLoadWrapper>
      </AsTile>
    </ContentBody>
  </div>
}
