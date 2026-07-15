
import React from 'react';
import {render} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import axios from 'axios';
import UpgradeTriggerModal from './UpgradeTriggerModal';

jest.mock('axios', () => ({
  post: jest.fn()
}));

describe('UpgradeTriggerModal', () => {
  it('renders the modal when showModal is true with the text', () => {
    const {getByText} = render(<UpgradeTriggerModal showModal={true} />);

    expect(getByText('Proceed with upgrade?')).toBeInTheDocument();
    expect(getByText('We highly recommend backing up your database before proceeding.')).toBeInTheDocument();
  });

  it('calls setShowModal with false when the cancel button is clicked', () => {
    const setShowModal = jest.fn();
    const {getByText} = render(<UpgradeTriggerModal showModal={true} setShowModal={setShowModal} />);

    userEvent.click(getByText('Cancel'));

    expect(setShowModal).toHaveBeenCalledWith(false);
    expect(axios.post).toHaveBeenCalledTimes(0);
  });

  it('calls put request when the save button is clicked', () => {
    const setShowModal = jest.fn();
    const {getByText, getByRole} = render(<UpgradeTriggerModal showModal={true} setShowModal={setShowModal} />);

    expect(getByText('Continue')).toBeDisabled();

    userEvent.click(getByRole('checkbox'));
    userEvent.click(getByText('Continue'));

    expect(getByText('Continue')).not.toBeDisabled();
    expect(setShowModal).toHaveBeenCalledWith(false);
    expect(axios.post).toHaveBeenCalledTimes(1);
    expect(axios.post).toHaveBeenCalledWith('service/rest/v1/clustered/upgrade-database-schema');
  });
});
