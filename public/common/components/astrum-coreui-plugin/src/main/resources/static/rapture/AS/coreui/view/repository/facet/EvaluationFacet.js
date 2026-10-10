
/**
 * Configuration for repository evaluation facet.
 */
Ext.define('AS.coreui.view.repository.facet.EvaluationFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-repository-evaluation-facet',
  requires: [
    'AS.I18n',
    'AS.util.Url'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {
        xtype: 'fieldset',
        cls: 'as-form-section',
        title: AS.I18n.get('EvaluationFacet_Title'),

        items: [
          {
            xtype: 'radiogroup',
            fieldLabel: AS.I18n.get('EvaluationFacet_Mode_FieldLabel'),
            allowBlank: false,
            columns: 1,
            vertical: true,
            itemId: 'evaluationModeRadioGroup',
            items: [
              {
                boxLabel: AS.I18n.get('EvaluationFacet_Mode_Inherit'),
                name: 'attributes.evaluation.mode',
                inputValue: 'INHERIT'
              },
              {
                boxLabel: AS.I18n.get('EvaluationFacet_Mode_Override'),
                name: 'attributes.evaluation.mode',
                inputValue: 'OVERRIDE'
              },
              {
                boxLabel: AS.I18n.get('EvaluationFacet_Mode_Disable'),
                name: 'attributes.evaluation.mode',
                inputValue: 'DISABLE'
              }
            ],
            listeners: {
              afterrender: function(radiogroup) {
                // Store original values from form record
                var form = radiogroup.up('form');
                if (form && form.getRecord) {
                  var record = form.getRecord();
                  if (record && record.data && record.data.attributes && record.data.attributes.evaluation) {
                    var evaluation = record.data.attributes.evaluation;
                    radiogroup.originalEvaluationValues = {
                      activityTimeFrame: evaluation.activityTimeFrame || '30',
                      artifactLatestVersions: evaluation.artifactLatestVersions || '1',
                      policyEvaluationStage: evaluation.policyEvaluationStage || 'build'
                    };
                  } else {
                    radiogroup.originalEvaluationValues = {
                      activityTimeFrame: '30',
                      artifactLatestVersions: '1',
                      policyEvaluationStage: 'build'
                    };
                  }
                }

                // Set default mode to INHERIT if no mode is set
                var currentValue = radiogroup.getValue();
                if (!currentValue || !currentValue['attributes.evaluation.mode']) {
                  radiogroup.setValue({'attributes.evaluation.mode': 'INHERIT'});
                }

                // Fetch global evaluation settings from API for INHERIT mode display
                me._fetchAndApplyGlobalSettings(radiogroup);
              },
              change: function(radiogroup, newValue) {

                var form = me.up('form');
                var activityTimeFrame = form.down('#evaluationActivityTimeFrame');
                var artifactLatestVersions = form.down('#evaluationArtifactLatestVersions');
                var policyEvaluationStage = form.down('#evaluationPolicyEvaluationStage');

                var mode = newValue['attributes.evaluation.mode'];
                var oldMode = radiogroup.lastMode;
                radiogroup.lastMode = mode;
                var enableFields = (mode === 'OVERRIDE');

                // Save current OVERRIDE values before leaving OVERRIDE mode
                if (oldMode === 'OVERRIDE' && mode !== 'OVERRIDE') {
                  radiogroup.savedOverrideValues = {
                    activityTimeFrame: activityTimeFrame ? activityTimeFrame.getValue() : null,
                    artifactLatestVersions: artifactLatestVersions ? artifactLatestVersions.getValue() : null,
                    policyEvaluationStage: policyEvaluationStage ? policyEvaluationStage.getValue() : null
                  };
                }

                // Enable/disable fields based on mode
                if (activityTimeFrame) {
                  activityTimeFrame.setDisabled(!enableFields);
                  activityTimeFrame.allowBlank = !enableFields;
                }
                if (artifactLatestVersions) {
                  artifactLatestVersions.setDisabled(!enableFields);
                  artifactLatestVersions.allowBlank = !enableFields;
                }
                if (policyEvaluationStage) {
                  policyEvaluationStage.setDisabled(!enableFields);
                  policyEvaluationStage.allowBlank = !enableFields;
                }

                // Set values based on mode
                if (mode === 'OVERRIDE') {
                  // Restore saved OVERRIDE values if returning to OVERRIDE, otherwise use global settings
                  var valuesToUse;
                  if (radiogroup.savedOverrideValues && (radiogroup.savedOverrideValues.activityTimeFrame || radiogroup.savedOverrideValues.artifactLatestVersions || radiogroup.savedOverrideValues.policyEvaluationStage)) {
                    // Restore previous OVERRIDE values
                    valuesToUse = radiogroup.savedOverrideValues;
                  } else {
                    // Use global settings (INHERIT values) for first-time OVERRIDE or from DISABLE
                    valuesToUse = radiogroup.globalEvaluationSettings || {
                      activityTimeFrame: '30',
                      artifactLatestVersions: '1',
                      policyEvaluationStage: 'build'
                    };
                  }
                  me._setFieldValues(form, valuesToUse, true);
                } else if (mode === 'INHERIT') {
                  // Fetch and show global settings values (disabled)
                  if (radiogroup.globalEvaluationSettings) {
                    // Use cached global settings
                    me._setFieldValues(form, radiogroup.globalEvaluationSettings, true);
                  } else {
                    // Fetch global settings if not already cached
                    me._fetchAndApplyGlobalSettings(radiogroup);
                  }
                } else if (mode === 'DISABLE') {
                  // Clear values for DISABLE mode
                  me._setFieldValues(form, {
                    activityTimeFrame: null,
                    artifactLatestVersions: null,
                    policyEvaluationStage: null
                  }, true);
                }
              }
            }
          },
          {
            xtype: 'combo',
            name: 'attributes.evaluation.activityTimeFrame',
            itemId: 'evaluationActivityTimeFrame',
            fieldLabel: AS.I18n.get('EvaluationFacet_TimeFrame_FieldLabel'),
            helpText: AS.I18n.get('EvaluationFacet_TimeFrame_HelpText'),
            disabled: true,
            allowBlank: true,
            editable: false,
            emptyText: AS.I18n.get('EvaluationFacet_TimeFrame_EmptyText'),
            store: [
              ['30', AS.I18n.get('EvaluationFacet_TimeFrame_30')],
              ['60', AS.I18n.get('EvaluationFacet_TimeFrame_60')],
              ['90', AS.I18n.get('EvaluationFacet_TimeFrame_90')]
            ],
            queryMode: 'local',
            validator: function() {
              var form = this.up('form');
              var radioGroup = form.down('#evaluationModeRadioGroup');
              var mode = radioGroup ? radioGroup.getValue()['attributes.evaluation.mode'] : null;

              if (mode === 'OVERRIDE' && !this.getValue()) {
                return AS.I18n.get('EvaluationFacet_Override_Required');
              }
              return true;
            }
          },
          {
            xtype: 'combo',
            name: 'attributes.evaluation.artifactLatestVersions',
            itemId: 'evaluationArtifactLatestVersions',
            fieldLabel: AS.I18n.get('EvaluationFacet_Versions_FieldLabel'),
            helpText: AS.I18n.get('EvaluationFacet_Versions_HelpText'),
            disabled: true,
            allowBlank: true,
            editable: false,
            emptyText: AS.I18n.get('EvaluationFacet_Versions_EmptyText'),
            store: [
              ['1', '1'],
              ['2', '2'],
              ['3', '3'],
              ['4', '4'],
              ['5', '5']
            ],
            queryMode: 'local',
            validator: function() {
              var form = this.up('form');
              var radioGroup = form.down('#evaluationModeRadioGroup');
              var mode = radioGroup ? radioGroup.getValue()['attributes.evaluation.mode'] : null;

              if (mode === 'OVERRIDE' && !this.getValue()) {
                return AS.I18n.get('EvaluationFacet_Override_Required');
              }
              return true;
            }
          },
          {
            xtype: 'combo',
            name: 'attributes.evaluation.policyEvaluationStage',
            itemId: 'evaluationPolicyEvaluationStage',
            fieldLabel: AS.I18n.get('EvaluationFacet_Stage_FieldLabel'),
            helpText: AS.I18n.get('EvaluationFacet_Stage_HelpText'),
            disabled: true,
            allowBlank: true,
            editable: false,
            emptyText: AS.I18n.get('EvaluationFacet_Stage_EmptyText'),
            store: [
              ['build', AS.I18n.get('EvaluationFacet_Stage_Build')],
              ['stage-release', AS.I18n.get('EvaluationFacet_Stage_StageRelease')],
              ['release', AS.I18n.get('EvaluationFacet_Stage_Release')],
              ['operate', AS.I18n.get('EvaluationFacet_Stage_Operate')]
            ],
            queryMode: 'local',
            validator: function() {
              var form = this.up('form');
              var radioGroup = form.down('#evaluationModeRadioGroup');
              var mode = radioGroup ? radioGroup.getValue()['attributes.evaluation.mode'] : null;

              if (mode === 'OVERRIDE' && !this.getValue()) {
                return AS.I18n.get('EvaluationFacet_Override_Required');
              }
              return true;
            }
          }
        ]
      }
    ];

    me.callParent();
  },

  /**
   * Helper method to set field values
   * @private
   */
  _setFieldValues: function(form, settings, clearInvalid) {
    var activityTimeFrame = form.down('#evaluationActivityTimeFrame');
    var artifactLatestVersions = form.down('#evaluationArtifactLatestVersions');
    var policyEvaluationStage = form.down('#evaluationPolicyEvaluationStage');

    if (activityTimeFrame) {
      activityTimeFrame.setValue(settings.activityTimeFrame);
      if (clearInvalid) {
        activityTimeFrame.clearInvalid();
      }
    }
    if (artifactLatestVersions) {
      artifactLatestVersions.setValue(settings.artifactLatestVersions);
      if (clearInvalid) {
        artifactLatestVersions.clearInvalid();
      }
    }
    if (policyEvaluationStage) {
      policyEvaluationStage.setValue(settings.policyEvaluationStage);
      if (clearInvalid) {
        policyEvaluationStage.clearInvalid();
      }
    }
  },

  /**
   * Helper method to fetch and apply global evaluation settings.
   *
   * ASTRUM-54168: the fetched settings are always cached on the radiogroup (they may be needed
   * later when the user switches to INHERIT), but are only applied to the form fields when the
   * current mode is INHERIT. Applying unconditionally clobbers the OVERRIDE values that
   * loadRecord placed on the fields moments earlier, which manifests as the persisted override
   * appearing to revert to the global values on the next visit to the settings page.
   *
   * @private
   */
  _fetchAndApplyGlobalSettings: function(radiogroup, callback) {
    var me = this;
    if (radiogroup.fetchingGlobalSettings) {
      return;
    }

    radiogroup.fetchingGlobalSettings = true;

    Ext.Ajax.request({
      url: AS.util.Url.urlOf('/service/rest/v1/evaluation/settings'),
      method: 'GET',
      success: function(response) {
        radiogroup.fetchingGlobalSettings = false;
        try {
          var globalSettings = (response.status === 204 || !response.responseText) ? null : Ext.decode(response.responseText);
          if (globalSettings) {
            var rawStage = globalSettings.policyEvaluationStage || 'build';
            var normalizedStage = rawStage.toLowerCase().replace(/_/g, '-');
            radiogroup.globalEvaluationSettings = {
              activityTimeFrame: String(globalSettings.activityTimeFrame != null ? globalSettings.activityTimeFrame : 30),
              artifactLatestVersions: String(globalSettings.artifactLatestVersions != null ? globalSettings.artifactLatestVersions : 1),
              policyEvaluationStage: normalizedStage
            };
          } else {
            radiogroup.globalEvaluationSettings = {
              activityTimeFrame: '30',
              artifactLatestVersions: '1',
              policyEvaluationStage: 'build'
            };
          }
        } catch (e) {
          // Use defaults if parsing fails
          radiogroup.globalEvaluationSettings = {
            activityTimeFrame: '30',
            artifactLatestVersions: '1',
            policyEvaluationStage: 'build'
          };
        }

        me._applyGlobalSettingsIfInherit(radiogroup);

        if (callback) {
          callback();
        }
      },
      failure: function() {
        radiogroup.fetchingGlobalSettings = false;
        // Use defaults if API call fails
        radiogroup.globalEvaluationSettings = {
          activityTimeFrame: '30',
          artifactLatestVersions: '1',
          policyEvaluationStage: 'build'
        };

        me._applyGlobalSettingsIfInherit(radiogroup);

        if (callback) {
          callback();
        }
      }
    });
  },

  /**
   * Apply cached global settings to the form fields, but only when the mode is INHERIT.
   * In OVERRIDE mode the fields already hold the persisted override values (placed there by
   * loadRecord) and must not be clobbered; in DISABLE mode the mode-change handler clears
   * them intentionally. See ASTRUM-54168.
   *
   * The form reference is resolved fresh via radiogroup.up('form') at apply-time rather than
   * captured earlier in _fetchAndApplyGlobalSettings, so a panel that was destroyed and
   * re-rendered while the async fetch was in flight cannot cause writes to a detached form.
   *
   * @private
   */
  _applyGlobalSettingsIfInherit: function(radiogroup) {
    if (!radiogroup) {
      return;
    }
    var form = radiogroup.up ? radiogroup.up('form') : null;
    if (!form) {
      return;
    }
    var currentValue = radiogroup.getValue();
    var currentMode = currentValue ? currentValue['attributes.evaluation.mode'] : null;
    if (currentMode === 'INHERIT' && radiogroup.globalEvaluationSettings) {
      this._setFieldValues(form, radiogroup.globalEvaluationSettings, true);
    }
  }
});
