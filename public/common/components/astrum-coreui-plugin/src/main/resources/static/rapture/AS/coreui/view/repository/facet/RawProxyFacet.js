
/*global Ext, AS*/

/**
 * Configuration for query parameter forwarding and write method support in raw proxy repositories.
 *
 * @since 3.90
 */
Ext.define('AS.coreui.view.repository.facet.RawProxyFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-repository-rawproxy-facet',
  requires: [
    'AS.I18n',
    'AS.Icons'
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
        title: AS.I18n.get('Repository_Facet_RawProxy_QueryParams_Title'),

        items: [
          // Checkbox with description below
          {
            xtype: 'checkbox',
            name: 'attributes.raw.forwardQueryParameters',
            itemId: 'forwardQueryParametersCheckbox',
            fieldLabel: AS.I18n.get('Repository_Facet_RawProxy_QueryParams_Checkbox'),
            boxLabel: AS.I18n.get('Repository_Facet_RawProxy_QueryParams_Description'),
            value: false,
            listeners: {
              change: function(checkbox, newValue) {
                var form = checkbox.up('as-coreui-repository-settings-form');
                if (form) {
                  var cachingWarning = form.down('#queryParamsCachingWarning');
                  var infoPanel = form.down('#queryParamsInfo');
                  var exclusionField = form.down('#excludedQueryParameters');

                  if (newValue) {
                    // Forwarding ENABLED
                    checkbox.setBoxLabel(AS.I18n.get('Repository_Facet_RawProxy_QueryParams_DescriptionEnabled'));
                    if (cachingWarning) {
                      cachingWarning.show();
                    }
                    if (infoPanel) {
                      infoPanel.show();
                    }
                    if (exclusionField) {
                      exclusionField.show();
                      exclusionField.enable();
                    }
                  } else {
                    // Forwarding DISABLED
                    checkbox.setBoxLabel(AS.I18n.get('Repository_Facet_RawProxy_QueryParams_Description'));
                    if (cachingWarning) {
                      cachingWarning.hide();
                    }
                    if (infoPanel) {
                      infoPanel.hide();
                    }
                    if (exclusionField) {
                      exclusionField.hide();
                      exclusionField.disable();
                    }
                  }
                }
              }
            }
          },

          // Warning panel for caching behavior (shown when enabled)
          {
            xtype: 'panel',
            itemId: 'queryParamsCachingWarning',
            ui: 'as-drilldown-message',
            cls: 'as-drilldown-warning',
            iconCls: AS.Icons.cls('drilldown-warning', 'x16'),
            hidden: true,
            title: AS.I18n.get('Repository_Facet_RawProxy_QueryParams_CachingWarningTitle'),
            html: AS.I18n.get('Repository_Facet_RawProxy_QueryParams_CachingWarning')
          },

          // Info panel (shown when enabled)
          {
            xtype: 'panel',
            itemId: 'queryParamsInfo',
            ui: 'as-drilldown-message',
            cls: 'as-drilldown-info',
            iconCls: AS.Icons.cls('drilldown-info', 'x16'),
            hidden: true,
            title: AS.I18n.get('Repository_Facet_RawProxy_QueryParams_InfoTitle'),
            html: AS.I18n.get('Repository_Facet_RawProxy_QueryParams_InfoDescription')
          },

          // Exclusion list (textfield with comma-separated values)
          {
            xtype: 'textfield',
            name: 'attributes.raw.excludedQueryParameters',
            itemId: 'excludedQueryParameters',
            fieldLabel: AS.I18n.get('Repository_Facet_RawProxy_QueryParams_ExcludedLabel'),
            helpText: AS.I18n.get('Repository_Facet_RawProxy_QueryParams_ExcludedHelp'),
            hidden: true,
            disabled: true,
            allowBlank: true,
            // Convert comma-separated string to array on submit
            getSubmitValue: function() {
              var value = this.getValue();
              if (!value || value.trim() === '') {
                return [];
              }
              return value.split(',').map(function(param) {
                return param.trim();
              }).filter(function(param) {
                return param.length > 0;
              });
            },
            // Bypass BasicForm toString coercion to preserve array type
            getModelData: function() {
              var data = {};
              data[this.getName()] = this.getSubmitValue();
              return data;
            },
            // Convert array to comma-separated string on load
            setValue: function(value) {
              if (Array.isArray(value)) {
                value = value.join(', ');
              }
              return Ext.form.field.Text.prototype.setValue.call(this, value);
            }
          }
        ]
      }
    ];

    me.callParent();
  }

});
