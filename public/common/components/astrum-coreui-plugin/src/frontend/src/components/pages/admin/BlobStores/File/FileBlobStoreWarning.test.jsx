
import React from 'react';
import {render, screen} from '@testing-library/react';
import {useActor} from '@xstate/react';
import {ExtJS} from '@globalart/astrum-ui-plugin';
import FileBlobStoreWarning from './FileBlobStoreWarning';

jest.mock('@xstate/react');
jest.mock('@globalart/astrum-ui-plugin', () => ({
  ExtJS: {
    state: jest.fn(),
  },
}));

describe('FileBlobStoreWarning', () => {
  const mockService = {};

  beforeEach(() => {
    jest.clearAllMocks();
  });

  function setupMocks({path, isClustered, workDirectory}) {
    useActor.mockReturnValue([
      {
        context: {
          data: {path},
        },
      },
    ]);

    ExtJS.state.mockReturnValue({
      getValue: jest.fn().mockReturnValue({
        isClustered,
        workDirectory,
      }),
    });
  }

  describe('when not clustered', () => {
    it('does not render warning for any path', () => {
      setupMocks({
        path: '/astrum-data/blobs',
        isClustered: false,
        workDirectory: '/astrum-data',
      });

      const {container} = render(<FileBlobStoreWarning service={mockService} />);
      expect(container.firstChild).toBeNull();
    });
  });

  describe('when clustered', () => {
    describe('relative paths', () => {
      it('shows warning for relative Unix path', () => {
        setupMocks({
          path: 'relative/path',
          isClustered: true,
          workDirectory: '/astrum-data',
        });

        render(<FileBlobStoreWarning service={mockService} />);
        expect(screen.getByText('High Availability Path Warning')).toBeInTheDocument();
      });

      it('shows warning for relative Windows path', () => {
        setupMocks({
          path: 'relative\\path',
          isClustered: true,
          workDirectory: 'C:\\astrum-data',
        });

        render(<FileBlobStoreWarning service={mockService} />);
        expect(screen.getByText('High Availability Path Warning')).toBeInTheDocument();
      });

      it('does not show warning for Unix absolute path outside work directory', () => {
        setupMocks({
          path: '/mnt/shared-storage/blobs',
          isClustered: true,
          workDirectory: '/astrum-data',
        });

        const {container} = render(<FileBlobStoreWarning service={mockService} />);
        expect(container.firstChild).toBeNull();
      });

      it('does not show warning for Windows absolute path outside work directory', () => {
        setupMocks({
          path: 'D:\\shared-storage\\blobs',
          isClustered: true,
          workDirectory: 'C:\\astrum-data',
        });

        const {container} = render(<FileBlobStoreWarning service={mockService} />);
        expect(container.firstChild).toBeNull();
      });
    });

    describe('paths under work directory', () => {
      it('shows warning for path directly under work directory', () => {
        setupMocks({
          path: '/astrum-data/blobs',
          isClustered: true,
          workDirectory: '/astrum-data',
        });

        render(<FileBlobStoreWarning service={mockService} />);
        expect(screen.getByText('High Availability Path Warning')).toBeInTheDocument();
      });

      it('shows warning for nested path under work directory', () => {
        setupMocks({
          path: '/astrum-data/blobs/default/content',
          isClustered: true,
          workDirectory: '/astrum-data',
        });

        render(<FileBlobStoreWarning service={mockService} />);
        expect(screen.getByText('High Availability Path Warning')).toBeInTheDocument();
      });

      it('does not show warning for sibling directory with prefix name', () => {
        setupMocks({
          path: '/astrum-data-backup/blobs',
          isClustered: true,
          workDirectory: '/astrum-data',
        });

        const {container} = render(<FileBlobStoreWarning service={mockService} />);
        expect(container.firstChild).toBeNull();
      });

      it('does not show warning when work directory name is prefix of path component', () => {
        setupMocks({
          path: '/astrum-data/information/blobs',
          isClustered: true,
          workDirectory: '/astrum-data/info',
        });

        const {container} = render(<FileBlobStoreWarning service={mockService} />);
        expect(container.firstChild).toBeNull();
      });

      it('shows warning when path is exactly under work directory subdirectory', () => {
        setupMocks({
          path: '/astrum-data/info/blobs',
          isClustered: true,
          workDirectory: '/astrum-data/info',
        });

        render(<FileBlobStoreWarning service={mockService} />);
        expect(screen.getByText('High Availability Path Warning')).toBeInTheDocument();
      });
    });

    describe('Windows paths', () => {
      it('shows warning for Windows path under work directory', () => {
        setupMocks({
          path: 'C:\\astrum-data\\blobs',
          isClustered: true,
          workDirectory: 'C:\\astrum-data',
        });

        render(<FileBlobStoreWarning service={mockService} />);
        expect(screen.getByText('High Availability Path Warning')).toBeInTheDocument();
      });

      it('does not show warning for Windows sibling directory', () => {
        setupMocks({
          path: 'C:\\astrum-data-backup\\blobs',
          isClustered: true,
          workDirectory: 'C:\\astrum-data',
        });

        const {container} = render(<FileBlobStoreWarning service={mockService} />);
        expect(container.firstChild).toBeNull();
      });

      it('normalizes mixed slashes in Windows paths', () => {
        setupMocks({
          path: 'C:/astrum-data/blobs',
          isClustered: true,
          workDirectory: 'C:\\astrum-data',
        });

        render(<FileBlobStoreWarning service={mockService} />);
        expect(screen.getByText('High Availability Path Warning')).toBeInTheDocument();
      });
    });

    describe('edge cases', () => {
      it('does not show warning for empty path', () => {
        setupMocks({
          path: '',
          isClustered: true,
          workDirectory: '/astrum-data',
        });

        const {container} = render(<FileBlobStoreWarning service={mockService} />);
        expect(container.firstChild).toBeNull();
      });

      it('does not show warning for null path', () => {
        setupMocks({
          path: null,
          isClustered: true,
          workDirectory: '/astrum-data',
        });

        const {container} = render(<FileBlobStoreWarning service={mockService} />);
        expect(container.firstChild).toBeNull();
      });

      it('does not show warning for undefined path', () => {
        setupMocks({
          path: undefined,
          isClustered: true,
          workDirectory: '/astrum-data',
        });

        const {container} = render(<FileBlobStoreWarning service={mockService} />);
        expect(container.firstChild).toBeNull();
      });

      it('does not show warning when work directory is empty', () => {
        setupMocks({
          path: '/astrum-data/blobs',
          isClustered: true,
          workDirectory: '',
        });

        const {container} = render(<FileBlobStoreWarning service={mockService} />);
        expect(container.firstChild).toBeNull();
      });

      it('handles work directory without trailing slash', () => {
        setupMocks({
          path: '/astrum-data/blobs',
          isClustered: true,
          workDirectory: '/astrum-data',
        });

        render(<FileBlobStoreWarning service={mockService} />);
        expect(screen.getByText('High Availability Path Warning')).toBeInTheDocument();
      });

      it('handles work directory with trailing slash', () => {
        setupMocks({
          path: '/astrum-data/blobs',
          isClustered: true,
          workDirectory: '/astrum-data/',
        });

        render(<FileBlobStoreWarning service={mockService} />);
        expect(screen.getByText('High Availability Path Warning')).toBeInTheDocument();
      });
    });

    describe('warning content', () => {
      it('displays the correct warning message', () => {
        setupMocks({
          path: 'relative/path',
          isClustered: true,
          workDirectory: '/astrum-data',
        });

        render(<FileBlobStoreWarning service={mockService} />);

        expect(screen.getByText('High Availability Path Warning')).toBeInTheDocument();
        expect(
          screen.getByText(
            /Using a relative path or a path under the working directory.*in HA mode can cause severe performance issues/
          )
        ).toBeInTheDocument();
      });
    });
  });
});
