

import { interpret } from 'xstate';

import { createEndpointDetailMachine } from '../endpointDetailMachine';

const ALL_PERMS = { hasSecurityDirectoryRead: true, hasGrantAccess: true };
const NO_PERMS = { hasSecurityDirectoryRead: false, hasGrantAccess: false };
const READ_ONLY = { hasSecurityDirectoryRead: true, hasGrantAccess: false };
const GRANT_ONLY = { hasSecurityDirectoryRead: false, hasGrantAccess: true };

describe('endpointDetailMachine', () => {
  it('shouldStartInTry', () => {
    const service = interpret(createEndpointDetailMachine(ALL_PERMS)).start();
    expect(service.getSnapshot().matches('try')).toBe(true);
    service.stop();
  });

  it('shouldSelectWhoWhenSecurityReadGranted', () => {
    const service = interpret(createEndpointDetailMachine(ALL_PERMS)).start();
    service.send({ type: 'SELECT_TAB', tab: 'who' });
    expect(service.getSnapshot().matches('who')).toBe(true);
    service.stop();
  });

  it('shouldNotSelectWhoWhenSecurityReadMissing', () => {
    const service = interpret(createEndpointDetailMachine(GRANT_ONLY)).start();
    service.send({ type: 'SELECT_TAB', tab: 'who' });
    expect(service.getSnapshot().matches('try')).toBe(true);
    service.stop();
  });

  it('shouldSelectGrantWhenGrantAccessGranted', () => {
    const service = interpret(createEndpointDetailMachine(ALL_PERMS)).start();
    service.send({ type: 'SELECT_TAB', tab: 'grant' });
    expect(service.getSnapshot().matches('grant')).toBe(true);
    service.stop();
  });

  it('shouldNotSelectGrantWhenGrantAccessMissing', () => {
    const service = interpret(createEndpointDetailMachine(READ_ONLY)).start();
    service.send({ type: 'SELECT_TAB', tab: 'grant' });
    expect(service.getSnapshot().matches('try')).toBe(true);
    service.stop();
  });

  it('shouldReturnToTryFromWhoOnSelectTry', () => {
    const service = interpret(createEndpointDetailMachine(ALL_PERMS)).start();
    service.send({ type: 'SELECT_TAB', tab: 'who' });
    service.send({ type: 'SELECT_TAB', tab: 'try' });
    expect(service.getSnapshot().matches('try')).toBe(true);
    service.stop();
  });

  it('shouldSwitchFromWhoToGrantDirectly', () => {
    const service = interpret(createEndpointDetailMachine(ALL_PERMS)).start();
    service.send({ type: 'SELECT_TAB', tab: 'who' });
    service.send({ type: 'SELECT_TAB', tab: 'grant' });
    expect(service.getSnapshot().matches('grant')).toBe(true);
    service.stop();
  });

  it('shouldSwitchFromGrantToWhoDirectly', () => {
    const service = interpret(createEndpointDetailMachine(ALL_PERMS)).start();
    service.send({ type: 'SELECT_TAB', tab: 'grant' });
    service.send({ type: 'SELECT_TAB', tab: 'who' });
    expect(service.getSnapshot().matches('who')).toBe(true);
    service.stop();
  });

  it('shouldAutoResetToTryWhenSecurityReadRevoked', () => {
    const service = interpret(createEndpointDetailMachine(ALL_PERMS)).start();
    service.send({ type: 'SELECT_TAB', tab: 'who' });
    service.send({ type: 'PERMISSIONS_UPDATED', hasSecurityDirectoryRead: false, hasGrantAccess: true });
    expect(service.getSnapshot().matches('try')).toBe(true);
    service.stop();
  });

  it('shouldAutoResetToTryWhenGrantAccessRevoked', () => {
    const service = interpret(createEndpointDetailMachine(ALL_PERMS)).start();
    service.send({ type: 'SELECT_TAB', tab: 'grant' });
    service.send({ type: 'PERMISSIONS_UPDATED', hasSecurityDirectoryRead: true, hasGrantAccess: false });
    expect(service.getSnapshot().matches('try')).toBe(true);
    service.stop();
  });

  it('shouldStayInWhoWhenUnrelatedPermissionChanges', () => {
    const service = interpret(createEndpointDetailMachine(ALL_PERMS)).start();
    service.send({ type: 'SELECT_TAB', tab: 'who' });
    service.send({ type: 'PERMISSIONS_UPDATED', hasSecurityDirectoryRead: true, hasGrantAccess: false });
    expect(service.getSnapshot().matches('who')).toBe(true);
    service.stop();
  });

  it('shouldAllowSelectingWhoAfterPermissionsUpdatedGrantsIt', () => {
    const service = interpret(createEndpointDetailMachine(NO_PERMS)).start();
    service.send({ type: 'SELECT_TAB', tab: 'who' });
    expect(service.getSnapshot().matches('try')).toBe(true);

    service.send({ type: 'PERMISSIONS_UPDATED', hasSecurityDirectoryRead: true, hasGrantAccess: false });
    service.send({ type: 'SELECT_TAB', tab: 'who' });
    expect(service.getSnapshot().matches('who')).toBe(true);
    service.stop();
  });

  it('shouldInitializeContextFromOptions', () => {
    const service = interpret(createEndpointDetailMachine(READ_ONLY)).start();
    const context = service.getSnapshot().context;
    expect(context.hasSecurityDirectoryRead).toBe(true);
    expect(context.hasGrantAccess).toBe(false);
    service.stop();
  });
});
