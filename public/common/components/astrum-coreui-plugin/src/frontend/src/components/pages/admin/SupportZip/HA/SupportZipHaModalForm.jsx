
import React from 'react';

import UIStrings from "../../../../../constants/UIStrings";
const {SUPPORT_ZIP: LABELS} = UIStrings;

import {
    AsModal, AsH2
} from "@globalart/react-shared-components";

import SupportZipForm from "../SupportZipForm";

export default function SupportZipHaModalForm({formParams, setFormParams, submitHandler, cancelHandler}) {
    return <AsModal onCancel={cancelHandler}>
        <AsModal.Header>
            <AsH2>
                <span>{LABELS.CREATE_SUPPORT_ZIP_MODAL_HEADER}</span>
            </AsH2>
        </AsModal.Header>

        <SupportZipForm
            params={formParams}
            setParams={setFormParams}
            submit={submitHandler}
            cancel={cancelHandler} />

    </AsModal>
}
