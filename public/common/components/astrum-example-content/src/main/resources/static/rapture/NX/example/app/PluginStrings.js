
/*global Ext, NX*/

/**
 * Example content plugin strings.
 */
Ext.define('NX.example.app.PluginStrings', {
  '@aggregate_priority': 90,

  singleton: true,
  requires: [
    'NX.I18n'
  ],

  keys: {
    Repository_Facet_ExampleFacet_Title: 'Example Settings',
  }
}, function(self) {
  NX.I18n.register(self);
});
