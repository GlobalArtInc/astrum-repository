
/*global Ext, AS*/

/*global AS, Ext, GlobalArt*/

/**
 * 'combo' factory.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.formfield.factory.FormfieldComboFactory', {
  singleton: true,
  alias: [
    'as.formfield.factory.combo',
    'as.formfield.factory.combobox',
    'as.formfield.factory.repo',
    'as.formfield.factory.repo-or-group',
    'as.formfield.factory.repo-target'
  ],
  requires: [
    'Ext.data.Store',
    'Ext.form.ComboBox',
    'AS.util.Url'
  ],
  mixins: {
    logAware: 'AS.LogAware'
  },

  /**
   * Creates a combo.
   * @param formField form field to create combo for
   * @returns {*} created combo (never null)
   */
  create: function (formField) { // NOSONAR

    var ST = Ext.data.SortTypes,
        item, filters,
        itemConfig = {
          xtype: 'combo',
          fieldLabel: formField.label,
          itemCls: formField.required ? 'required-field' : '',
          helpText: formField.helpText,
          name: formField.id,
          displayField: 'name',
          valueField: 'id',
          editable: false,
          forceSelection: true,
          queryMode: 'local',
          triggerAction: 'all',
          emptyText: AS.I18n.get('Form_Select_EmptyText'),
          selectOnFocus: false,
          allowBlank: !formField.required
        };

    if (formField.allowAutocomplete) {
      itemConfig.editable = true;
      itemConfig.mode = 'remote';
      itemConfig.queryMode = 'remote';
      itemConfig.autoSelect = false;
      itemConfig.hideTrigger = false;
      itemConfig.typeAhead = true;
      itemConfig.minChars = 2;
    }

    if (formField.initialValue != null) {
      itemConfig.listeners = {
        afterrender: function() {
          var me = this;
          if (!me.getValue()) {
            me.setValue(formField.initialValue);
          }
        }
      };
    }
    if (formField['storeApi']) {
      if (formField['storeFilters']) {
        filters = [];
        Ext.Object.each(formField['storeFilters'], function (key, value) {
          filters.push({ property: key, value: value });
        });
      }
      itemConfig.store = Ext.create('Ext.data.Store', {
        proxy: {
          type: 'direct',
          api: {
            read: 'AS.direct.' + formField['storeApi']
          },
          reader: {
            type: 'json',
            rootProperty: 'data',
            idProperty: formField['idMapping'] || 'id',
            successProperty: 'success'
          }
        },

        fields: [
          {name: 'id', mapping: formField['idMapping'] || 'id'},
          {name: 'name', mapping: formField['nameMapping'] || 'name', sortType: ST.asUCString},
          {name: 'sortOrder', sortType: ST.asInt}
        ],

        filters: filters,
        sortOnLoad: true,
        sorters: [{ property: 'sortOrder', direction: 'DESC' }, { property: formField['attributes']['sortProperty'] || 'name', direction: 'ASC' }],
        remoteFilter: true,
        autoLoad: true
      });
    } else {
      itemConfig.store = Ext.create('Ext.data.Store', {
        fields: [
          {name: 'id', mapping: formField['idMapping'] || 'id'},
          {name: 'name', mapping: formField['nameMapping'] || 'name', sortType: ST.asUCString},
          {name: 'sortOrder', sortType: ST.asInt}
        ],

        sortOnLoad: true,
        sorters: [{ property: 'sortOrder', direction: 'DESC' }, { property: formField['attributes']['sortProperty'] || 'name', direction: 'ASC' }],
        remoteFilter: true,
        autoLoad: false
      });
    }
    item = Ext.create('Ext.form.ComboBox', itemConfig);
    Ext.override(item, {
      /**
       * Avoid value being set to null by combobox in case that store was not already loaded.
       */
      setValue: function (value) {
        var me = this;
        me.callParent(arguments);
        if (me.getValue() === null) {
          me.value = value;
        }
      }
    });
    return item;
  }
});
