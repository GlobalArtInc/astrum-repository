
/*global Ext, AS*/

/**
 * Asset info panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.component.AssetAttributes', {
  extend: 'Ext.grid.Panel',
  alias: 'widget.as-coreui-component-assetattributes',
  requires: [
    'AS.I18n'
  ],

  statics: {
    HIDDEN_FACETS: ['npm_rev', 'upstream_globalart_filtered_versions']
  },

  cls: 'as-coreui-component-assetattributes',

  manageHeight: false,

  disableSelection: true,
  rowLines: false,

  /**
   * model to display
   */
  assetModel: null,
  viewConfig: {
    enableTextSelection: true
  },
  itemId: 'attributeGrid',
  store: Ext.create('Ext.data.Store', {
    fields: ['facet', 'label', 'value'],
    groupField: 'facet'
  }),
  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.columns = {
      items: [
        {
          text: AS.I18n.get('AssetAttributes_Facet_Header'),
          hidden: true,
          dataIndex: 'facet'
        },
        {
          text: AS.I18n.get('AssetAttributes_Label_Header'),
          flex: 1,
          dataIndex: 'label',
          renderer: function(value, metaData) {
            metaData.tdAttr = 'data-qtip="' + value + '"';
            return value;
          }
        },
        {
          text: AS.I18n.get('AssetAttributes_Value_Header'),
          flex: 2,
          dataIndex: 'value',
          renderer: function (val) {
            var attributeValue = val;
            if (attributeValue == 0 || attributeValue == false) {
              // for correct html rendering '0' and 'false' values must be converted to string
              attributeValue = String(attributeValue);
            }

            return Ext.DomHelper.markup({
              tag: 'div',
              cls: 'attribute-value',
              html: Ext.String.htmlEncode(attributeValue)
            });
          }
        }
      ]
    };

    me.callParent();
  },

  hideHeaders: true,
  features: [
    {
      ftype: 'grouping',
      groupHeaderTpl: '{name:capitalize}'
    }
  ],

  /**
   * @public
   * @param {Object} assetModel the asset to display
   */
  setAssetModel: function(assetModel) {
    var me = this,
      store = me.getStore();
    me.assetModel = assetModel;

    // update the grid attribute data
    store.removeAll();
    Ext.iterate(me.assetModel.get('attributes'), function(facet, facetValues) {
      if (Ext.Array.contains(me.self.HIDDEN_FACETS, facet)) {
        return;
      }
      Ext.iterate(facetValues, function(key, value) {
        store.add({facet: facet, label: key, value: (Ext.isObject(value) || Ext.isArray(value)) ? Ext.encode(value) : value});
      });
    });
  }
});
