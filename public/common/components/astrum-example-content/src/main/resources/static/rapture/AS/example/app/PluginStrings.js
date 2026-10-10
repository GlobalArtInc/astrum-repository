
/*global Ext, AS*/

/**
 * Example content plugin strings.
 */
Ext.define('AS.example.app.PluginStrings', {
  '@aggregate_priority': 90,

  singleton: true,
  requires: [
    'AS.I18n'
  ],

  keys: {
    Repository_Facet_ExampleFacet_Title: 'Example Settings',
  }
}, function(self) {
  AS.I18n.register(self);
});
