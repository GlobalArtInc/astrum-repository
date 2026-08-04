
import React from 'react';
import {render} from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import MasterDetail from './MasterDetail';
import Master from './Master';
import Detail from './Detail';

import ExtJS from '../../../interface/ExtJS';

jest.mock('@uirouter/react', () => ({
  useRouter: jest.fn(() => ({
    urlService: {
      path: jest.fn(() => 'admin')
    }
  })),
}));

jest.mock('../../../interface/ExtJS');

describe('MasterDetail', () => {
  it('renders the master view when at the root', () => {
    ExtJS.useHistory.mockReturnValue({location: {pathname: ''}});
    const {queryByTestId} = render(
        <MasterDetail path="admin">
          <Master><TestMaster /></Master>
          <Detail><TestDetail /></Detail>
        </MasterDetail>
    );

    expect(queryByTestId('master')).toBeInTheDocument();
    expect(queryByTestId('detail')).not.toBeInTheDocument();
  });

  it('renders the detail view when on a child path', () => {
    ExtJS.useHistory.mockReturnValue({location: {pathname: ':itemId'}});
    const {queryByTestId} = render(
        <MasterDetail path="admin">
          <Master><TestMaster /></Master>
          <Detail><TestDetail /></Detail>
        </MasterDetail>
    );

    expect(queryByTestId('master')).not.toBeInTheDocument();
    expect(queryByTestId('detail')).toBeInTheDocument();
  });

  it('renders the detail view when onCreate is called', () => {
    ExtJS.useHistory.mockReturnValue({location: {pathname: ''}});
    const {getByTestId, queryByTestId} = render(
        <MasterDetail path="admin">
          <Master><TestMaster /></Master>
          <Detail><TestDetail /></Detail>
        </MasterDetail>
    );

    expect(queryByTestId('master')).toBeInTheDocument();
    userEvent.click(getByTestId('create'));

    expect(queryByTestId('master')).not.toBeInTheDocument();
    expect(queryByTestId('detail')).toBeInTheDocument();
  });

  it('does not render if the path does not match', () => {
    ExtJS.useHistory.mockReturnValue({location: {pathname: ''}});
    const {queryByTestId} = render(
      <MasterDetail path="something">
        <Master><TestMaster /></Master>
        <Detail><TestDetail /></Detail>
      </MasterDetail>
    );

    expect(queryByTestId('master')).not.toBeInTheDocument();
    expect(queryByTestId('detail')).not.toBeInTheDocument();
  });

  describe('itemId encoding from pathname', () => {
    it('properly encodes itemId when pathname contains slashes', () => {
      ExtJS.useHistory.mockReturnValue({location: {pathname: ':tasks/example.log'}});
      let capturedItemId;
      const TestDetailCapture = ({itemId}) => {
        capturedItemId = itemId;
        return <div data-testid="detail">{itemId}</div>;
      };

      render(
        <MasterDetail path="admin">
          <Master><TestMaster /></Master>
          <Detail><TestDetailCapture /></Detail>
        </MasterDetail>
      );

      expect(capturedItemId).toBe('tasks%2Fexample.log');
    });

    it('properly encodes itemId with question marks', () => {
      ExtJS.useHistory.mockReturnValue({location: {pathname: ':file?name.txt'}});
      let capturedItemId;
      const TestDetailCapture = ({itemId}) => {
        capturedItemId = itemId;
        return <div data-testid="detail">{itemId}</div>;
      };

      render(
        <MasterDetail path="admin">
          <Master><TestMaster /></Master>
          <Detail><TestDetailCapture /></Detail>
        </MasterDetail>
      );

      expect(capturedItemId).toBe('file%3Fname.txt');
    });

    it('properly encodes itemId with ampersands', () => {
      ExtJS.useHistory.mockReturnValue({location: {pathname: ':file&name.txt'}});
      let capturedItemId;
      const TestDetailCapture = ({itemId}) => {
        capturedItemId = itemId;
        return <div data-testid="detail">{itemId}</div>;
      };

      render(
        <MasterDetail path="admin">
          <Master><TestMaster /></Master>
          <Detail><TestDetailCapture /></Detail>
        </MasterDetail>
      );

      expect(capturedItemId).toBe('file%26name.txt');
    });

    it('properly encodes itemId with spaces', () => {
      ExtJS.useHistory.mockReturnValue({location: {pathname: ':file name.txt'}});
      let capturedItemId;
      const TestDetailCapture = ({itemId}) => {
        capturedItemId = itemId;
        return <div data-testid="detail">{itemId}</div>;
      };

      render(
        <MasterDetail path="admin">
          <Master><TestMaster /></Master>
          <Detail><TestDetailCapture /></Detail>
        </MasterDetail>
      );

      expect(capturedItemId).toBe('file%20name.txt');
    });

    it('properly encodes itemId with hash symbols', () => {
      ExtJS.useHistory.mockReturnValue({location: {pathname: ':file#name.txt'}});
      let capturedItemId;
      const TestDetailCapture = ({itemId}) => {
        capturedItemId = itemId;
        return <div data-testid="detail">{itemId}</div>;
      };

      render(
        <MasterDetail path="admin">
          <Master><TestMaster /></Master>
          <Detail><TestDetailCapture /></Detail>
        </MasterDetail>
      );

      expect(capturedItemId).toBe('file%23name.txt');
    });

    it('properly encodes itemId with plus signs', () => {
      ExtJS.useHistory.mockReturnValue({location: {pathname: ':file+name.txt'}});
      let capturedItemId;
      const TestDetailCapture = ({itemId}) => {
        capturedItemId = itemId;
        return <div data-testid="detail">{itemId}</div>;
      };

      render(
        <MasterDetail path="admin">
          <Master><TestMaster /></Master>
          <Detail><TestDetailCapture /></Detail>
        </MasterDetail>
      );

      expect(capturedItemId).toBe('file%2Bname.txt');
    });

    it('handles already-encoded pathname without double-encoding', () => {
      ExtJS.useHistory.mockReturnValue({location: {pathname: ':tasks%2Fexample.log'}});
      let capturedItemId;
      const TestDetailCapture = ({itemId}) => {
        capturedItemId = itemId;
        return <div data-testid="detail">{itemId}</div>;
      };

      render(
        <MasterDetail path="admin">
          <Master><TestMaster /></Master>
          <Detail><TestDetailCapture /></Detail>
        </MasterDetail>
      );

      // Should decode then re-encode to normalize: tasks%2Fexample.log -> tasks/example.log -> tasks%2Fexample.log
      expect(capturedItemId).toBe('tasks%2Fexample.log');
    });
  });

  describe('onEdit encoding', () => {
    it('encodes itemId with slashes before navigation', () => {
      ExtJS.useHistory.mockReturnValue({location: {pathname: ''}});
      const originalHash = window.location.hash;

      const {getByTestId} = render(
        <MasterDetail path="admin">
          <Master><TestMaster /></Master>
          <Detail><TestDetail /></Detail>
        </MasterDetail>
      );

      // Simulate clicking edit with an itemId containing a slash
      const editButton = getByTestId('edit-tasks/example.log');
      userEvent.click(editButton);

      // Verify hash was set with properly encoded itemId
      expect(window.location.hash).toBe('#admin:tasks%2Fexample.log');

      window.location.hash = originalHash;
    });

    it('handles already-encoded itemId without double-encoding', () => {
      ExtJS.useHistory.mockReturnValue({location: {pathname: ''}});
      const originalHash = window.location.hash;

      const {getByTestId} = render(
        <MasterDetail path="admin">
          <Master><TestMaster /></Master>
          <Detail><TestDetail /></Detail>
        </MasterDetail>
      );

      // Simulate clicking edit with an already-encoded itemId
      const editButton = getByTestId('edit-tasks%2Fexample.log');
      userEvent.click(editButton);

      // Should not double-encode - result should be same as previous test
      expect(window.location.hash).toBe('#admin:tasks%2Fexample.log');

      window.location.hash = originalHash;
    });

    it('encodes itemId with multiple special characters', () => {
      ExtJS.useHistory.mockReturnValue({location: {pathname: ''}});
      const originalHash = window.location.hash;

      const {getByTestId} = render(
        <MasterDetail path="admin">
          <Master><TestMaster /></Master>
          <Detail><TestDetail /></Detail>
        </MasterDetail>
      );

      // Test with spaces, ampersands, and question marks
      const editButton = getByTestId('edit-file name & test?.log');
      userEvent.click(editButton);

      expect(window.location.hash).toBe('#admin:file%20name%20%26%20test%3F.log');

      window.location.hash = originalHash;
    });
  });
});

// These components remove the warnings about the Master/Detail props not being used and provide hooks for validation
function TestMaster({onCreate, onEdit}) {
  return (
    <div data-testid="master">
      <button data-testid="create" onClick={onCreate} />
      {/* Buttons to test onEdit with various itemIds */}
      {onEdit && (
        <>
          <button data-testid="edit-tasks/example.log" onClick={() => onEdit('tasks/example.log')} />
          <button data-testid="edit-tasks%2Fexample.log" onClick={() => onEdit('tasks%2Fexample.log')} />
          <button data-testid="edit-file name & test?.log" onClick={() => onEdit('file name & test?.log')} />
        </>
      )}
    </div>
  );
}

function TestDetail({itemId}) {
  return <div data-testid="detail">${itemId}</div>;
}
