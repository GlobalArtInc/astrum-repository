
/*global Ext, AS, Image*/

/**
 * Dependency snippets controller.
 *
 * @since 3.15
 */
Ext.define('AS.controller.DependencySnippet', {
  extend: 'AS.app.Controller',
  requires: [
    'Ext.XTemplate'
  ],

  models: [
    'DependencySnippet'
  ],

  stores: [
    'DependencySnippet'
  ],

  /**
   * Add a new dependency snippet generator
   *
   * @public
   */
  addDependencySnippetGenerator: function(format, snippetGenerator) {
    this.getStore('DependencySnippet').add({
      format: format,
      snippetGenerator: snippetGenerator
    });
  },

  /**
   * Retrieve dependency snippets for a given format, component and asset.
   * Leave assetModel undefined if requesting snippets for a component.
   *
   * @public
   */
  getDependencySnippets: function(format, componentModel, assetModel) {
    var store = this.getStore('DependencySnippet');
    var dependencySnippets = [];

    store.queryRecordsBy(function(record) {
      return format === record.get('format');
    }).forEach(function(record) {
      var snippets = record.get('snippetGenerator')(componentModel, assetModel);
      Array.prototype.push.apply(dependencySnippets, snippets);
    });

    return dependencySnippets;
  }
});
