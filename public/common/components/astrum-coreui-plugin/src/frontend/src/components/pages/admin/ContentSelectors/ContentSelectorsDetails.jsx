
import React, {useCallback, useEffect} from 'react';
import {useMachine} from '@xstate/react';

import {ExtJS} from '@globalart/astrum-ui-plugin';

import {
  ContentBody,
  PageHeader,
  PageTitle,
  Page,
} from '@globalart/astrum-ui-plugin';

import ContentSelectorsFormMachine from './ContentSelectorsFormMachine';
import ContentSelectorsForm from './ContentSelectorsForm';
import ContentSelectorsReadOnly from './ContentSelectorsReadOnly';

import UIStrings from '../../../../constants/UIStrings';
import {useCurrentStateAndParams, useRouter} from "@uirouter/react";
import {ROUTE_NAMES} from "../../../../routerConfig/routeNames/routeNames";

const ADMIN = ROUTE_NAMES.ADMIN;

export default function ContentSelectorDetails() {
  const router = useRouter();
  const {state: routerState, params} = useCurrentStateAndParams();
  const onDone = useCallback(() => router.stateService.go(ADMIN.REPOSITORY.SELECTORS.LIST), [router]);
  const itemId = params?.itemId;

  const [state, , service] = useMachine(ContentSelectorsFormMachine, {
    context: {
      pristineData: {
        name: itemId
      }
    },

    actions: {
      onSaveSuccess: onDone,
      onDeleteSuccess: onDone
    },

    devTools: true
  });

  useEffect(() => {
    // we should not render edit form if itemId is not provided
    if (routerState.name === ADMIN.REPOSITORY.SELECTORS.EDIT && !itemId) {
      router.stateService.go(ROUTE_NAMES.MISSING_ROUTE)
    }

  }, [itemId]);

  const {pristineData} = state.context;

  const canUpdate = ExtJS.checkPermission('astrum:selectors:update');

  return <Page className="as-content-selectors">
    <PageHeader>
      <PageTitle text={Boolean(pristineData.name) ?
          UIStrings.CONTENT_SELECTORS.EDIT_TITLE(pristineData.name) :
          UIStrings.CONTENT_SELECTORS.MENU.text}/>
    </PageHeader>
    <ContentBody>
      {(canUpdate || !itemId)
          ? <ContentSelectorsForm service={service} onDone={onDone}/>
          : <ContentSelectorsReadOnly service={service} onDone={onDone}/>
      }
    </ContentBody>
  </Page>;
}
