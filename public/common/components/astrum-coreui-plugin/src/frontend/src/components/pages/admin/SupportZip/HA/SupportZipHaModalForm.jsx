
import React from 'react';

import UIStrings from "../../../../../constants/UIStrings";
const {SUPPORT_ZIP: LABELS} = UIStrings;

import {
    NxModal, NxH2
} from "@globalart/react-shared-components";

import SupportZipForm from "../SupportZipForm";

export default function SupportZipHaModalForm({formParams, setFormParams, submitHandler, cancelHandler}) {
    return <NxModal onCancel={cancelHandler}>
        <NxModal.Header>
            <NxH2>
                <span>{LABELS.CREATE_SUPPORT_ZIP_MODAL_HEADER}</span>
            </NxH2>
        </NxModal.Header>

        <SupportZipForm
            params={formParams}
            setParams={setFormParams}
            submit={submitHandler}
            cancel={cancelHandler} />

    </NxModal>
}
