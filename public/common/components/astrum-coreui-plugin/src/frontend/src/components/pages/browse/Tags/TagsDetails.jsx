
import React from 'react';
import {useMachine} from '@xstate/react';

import {ReadOnlyField} from '@globalart/astrum-ui-plugin';
import {
  NxLoadWrapper,
  NxTile,
  NxH2,
  NxTextLink,
  NxBackButton,
  NxReadOnly,
  NxCopyToClipboard
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
      <NxBackButton href="#browse/tags" text={TAGS.DETAILS.BACK_TO_TAGS_TABLE}/>
    </div>
    <ContentBody>
      <NxTile>
        <NxLoadWrapper retryHandler={retry} loading={isLoading} error={hasLoadError}>
          {() => (
            <>
              <NxTile.Header>
                <NxTile.HeaderTitle>
                  <NxH2>{tagData.name} {TAGS.DETAILS.TILE_HEADER}</NxH2>
                </NxTile.HeaderTitle>
                <NxTile.HeaderActions>
                  <NxTextLink href={'#browse/search/custom/=' + encodeURIComponent(`tags="${tagData.name}"`)}>{TAGS.DETAILS.FIND_TAGGED}</NxTextLink>
                </NxTile.HeaderActions>
              </NxTile.Header>
              <NxTile.Content>
                <NxReadOnly>
                  <ReadOnlyField label={TAGS.DETAILS.FIRST_CREATED} value={new Date(tagData.firstCreated).toLocaleString()}/>
                  <ReadOnlyField label={TAGS.DETAILS.LAST_UPDATED} value={new Date(tagData.lastUpdated).toLocaleString()}/>
                </NxReadOnly>
                <NxCopyToClipboard 
                  label={TAGS.DETAILS.ATTRIBUTES}
                  content={JSON.stringify(tagData.attributes, null, 2)} 
                />
              </NxTile.Content>
            </>
          )}
        </NxLoadWrapper>
      </NxTile>
    </ContentBody>
  </div>
}
