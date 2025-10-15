
var TestClasses = {};

var Ext = {
  define: function(className, implementationObject) {
    TestClasses[className] = implementationObject;
  }
};

function MockComponent(properties) {
  this.properties = properties;
}
MockComponent.prototype.get = function(propertyName) {
  return this.properties[propertyName];
};
