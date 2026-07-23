
import React from 'react';
import {act} from 'react-dom/test-utils';
import {render} from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import SupportZipResponse from './SupportZipResponse';

describe('SupportZipResponse', function() {
  const renderView = async (response, download) => {
    var selectors;

    await act(async () => {
      selectors = render(<SupportZipResponse response={response} download={download}/>);
    });

    return selectors;
  };

  it('renders support zip response', async function() {
    const response = {
      "name": "support-20200527-095617-6.zip",
      "size": "281152",
      "file": "/Users/ataylor/dev/globalart/astrum-internal/target/globalart-work/astrum3/downloads/support-20200527-095617-6.zip",
      "truncated": "false"
    };

    const {container} = await renderView(response);

    expect(container.querySelector('input[id="name"]')).toHaveValue("support-20200527-095617-6.zip");
    expect(container.querySelector('input[id="size"]')).toHaveValue("281152");
    expect(container.querySelector('input[id="file"]')).toHaveValue("/Users/ataylor/dev/globalart/astrum-internal/target/globalart-work/astrum3/downloads/support-20200527-095617-6.zip");
  });

  it('downloads', async function() {
    const response = {
      "name": "support-20200527-095617-6.zip",
      "size": "281152",
      "file": "/Users/ataylor/dev/globalart/astrum-internal/target/globalart-work/astrum3/downloads/support-20200527-095617-6.zip",
      "truncated": "false"
    };

    const download = jest.fn();
    const {container} = await renderView(response, download);
    const downloadButton = container.querySelector('button[type=submit]');

    userEvent.click(downloadButton);

    expect(download).toBeCalledWith(expect.any(Object), 'support-20200527-095617-6.zip');
  });
});
