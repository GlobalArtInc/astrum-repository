# ExtJS Shared Mock

This directory contains shared mocks for the astrum-ui-plugin module to ensure consistent testing across all test files.

## ExtJS Mock

The `ExtJS.js` mock provides default implementations for commonly used ExtJS methods. It automatically:

- Mocks all static methods with sensible defaults
- Sets up `window.dirty` tracking for `setDirtyStatus`
- Provides common return values for state methods
- Returns resolved promises for async operations

### Usage

To use the shared ExtJS mock in your test file:

```javascript
// Add this line to your test file
jest.mock('../../../interface/ExtJS');

// Import ExtJS normally
import ExtJS from '../../../interface/ExtJS';

// In your test, customize specific methods as needed
beforeEach(() => {
  // Use jest-when directly with ExtJS methods for clean, readable tests
  when(ExtJS.state().getValue)
    .calledWith('some.config.key')
    .mockReturnValue('expected-value');
    
  when(ExtJS.state().getValue)
    .calledWith('another.config.key')
    .mockReturnValue('another-value');
});
```

### Benefits

1. **Consistency** - All tests use the same base mock setup
2. **Maintainability** - Changes to ExtJS interface only require updating one file
3. **Simplicity** - Tests can focus on behavior rather than mock setup
4. **Flexibility** - Individual tests can still override specific methods as needed

### Customization

Each test can customize the mock by:
- Using `jest.spyOn()` to override specific methods
- Using `mockReturnValue()` or `mockImplementation()` to change behavior
- Using `jest-when` for conditional returns based on parameters

The shared mock provides sensible defaults while allowing tests to customize what they need.