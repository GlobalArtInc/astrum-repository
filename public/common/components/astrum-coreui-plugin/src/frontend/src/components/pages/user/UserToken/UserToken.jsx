
import React from 'react';
import {useMachine, useActor} from '@xstate/react';
import {faKey, faLock} from '@fortawesome/free-solid-svg-icons';
import {isNil} from 'ramda';
import {
  ContentBody,
  DateUtils,
  Page,
  PageHeader,
  PageTitle,
  Section,
  SectionFooter
} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsButtonBar,
  AsFontAwesomeIcon,
  AsH2,
  AsErrorAlert,
  AsInfoAlert,
  AsLoadWrapper,
  AsReadOnly,
  useToggle
} from '@globalart/react-shared-components';

 import UIStrings from '../../../../constants/UIStrings';
import UserTokenMachine from './UserTokenMachine';
import UserTokenDetails from './UserTokenDetails';
import './UserToken.scss';

const {
  USER_TOKEN: {
    MENU,
    CAPTION,
    LABELS, MESSAGES,
    BUTTONS,
    USER_TOKEN_STATUS
  }
} = UIStrings;

export default function UserToken() {
  const service = useMachine(UserTokenMachine, {devTools: true})[2];

  return <UserTokenForm service={service}/>;
}

export function UserTokenForm({service}) {
  const [state, send] = useActor(service);

  const {data, error, token} = state.context;
  const isLoading = !(state.matches('loaded') || state.matches('showToken'));
  const expirationTimestamp = data?.expirationTimeTimestamp;
  const isExpired = !isNil(error);
  const [isOpen, dismiss] = useToggle(true);
  const showUserTokenStatus = !isNil(expirationTimestamp) || isExpired;

  function onAccessTokenClick() {
    send({type: 'ACCESS'});
  }

  function onResetTokenClick() {
    send({type: 'RESET'});
  }

  function onGenerateTokenClick() {
    send('GENERATE');
  }
  function onCloseClick() {
    send({type: 'HIDE'});
  }

  function retry() {
    send({type: 'RETRY'});
  }

  return <Page>
    <PageHeader><PageTitle icon={faKey} {...MENU}/></PageHeader>
    <ContentBody className='as-usertoken-current'>
      {state.matches('showToken') && <UserTokenDetails userToken={token} onCloseClick={onCloseClick}/>}
      <Section>
        <AsLoadWrapper loading={isLoading} retryHandler={retry}>
          <AsH2>{CAPTION}</AsH2>
          <AsInfoAlert>{LABELS.NOTE}</AsInfoAlert>
          {showUserTokenStatus &&
            <AsReadOnly>
              <AsReadOnly.Label>{USER_TOKEN_STATUS.TEXT}</AsReadOnly.Label>
              <AsReadOnly.Data className="as-sub-label">{USER_TOKEN_STATUS.DESCRIPTION}</AsReadOnly.Data>
              <AsReadOnly.Data>
                {USER_TOKEN_STATUS.TIMESTAMP_TEXT(isExpired)}
                {isExpired && isOpen && <AsErrorAlert onClose={dismiss}>{error}</AsErrorAlert>}
                {!isExpired && DateUtils.prettyDateTimeLong(new Date(parseInt(expirationTimestamp)))}
              </AsReadOnly.Data>
            </AsReadOnly>
          }
          <SectionFooter>
            <AsButtonBar>
              <AsButton onClick={onResetTokenClick}>
                <AsFontAwesomeIcon icon={faLock}/>
                <span>{BUTTONS.RESET}</span>
              </AsButton>
              {isExpired && <AsButton variant='primary' onClick={onGenerateTokenClick}>
                <span>{BUTTONS.GENERATE}</span>
              </AsButton>}
              {!isExpired && <AsButton variant='primary' onClick={onAccessTokenClick}>
                <span>{BUTTONS.ACCESS}</span>
              </AsButton>}
            </AsButtonBar>
          </SectionFooter>
        </AsLoadWrapper>
      </Section>
    </ContentBody>
  </Page>;
}
