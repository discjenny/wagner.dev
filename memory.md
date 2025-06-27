# Memory Optimization Opportunities

## 🔥 **Priority 1: High Impact**

### 1. Icon HashMap Recreation
**File**: `server/src/routes/icons.rs`  
**Problem**: HashMap created on every icon request
```rust
// Current - line 32
fn get_svg_content(icon_name: &str) -> Option<&'static str> {
    let mut icons = HashMap::new();  // 🚨 Heap allocation on every call
```

**Solution**: Use static HashMap with `once_cell`
```rust
use once_cell::sync::Lazy;

static ICONS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut icons = HashMap::new();
    icons.insert("sun", r#"<svg...>"#);
    icons.insert("moon", r#"<svg...>"#);
    icons
});

fn get_svg_content(icon_name: &str) -> Option<&'static str> {
    ICONS.get(icon_name).copied()
}
```

### 2. Repeated Default Theme Allocations
**Files**: `server/src/routes/themes.rs`, `server/src/routes/pages.rs`  
**Problem**: Multiple `"dark".to_string()` calls throughout codebase

**Locations**:
- `themes.rs:31` - `Err(_) => "dark".to_string(),`
- `themes.rs:49` - `theme: "dark".to_string(),`
- `themes.rs:58` - `theme: "dark".to_string(),`
- `themes.rs:73` - `theme: "dark".to_string(),`
- `themes.rs:99` - `Ok("dark".to_string())`
- `pages.rs:29` - `.unwrap_or_else(|_| "dark".to_string());`
- `pages.rs:39` - `.unwrap_or_else(|_| "dark".to_string());`
- `pages.rs:64` - `Ok("dark".to_string())`

**Solution**: Use constant
```rust
const DEFAULT_THEME: &str = "dark";

// Then use:
Err(_) => DEFAULT_THEME.to_string(),
// Or better: return &str where possible, convert only at API boundaries
```

## ⚡ **Priority 2: Medium Impact**

### 3. Default Classes Function Heap Allocation
**File**: `server/src/routes/icons.rs:15`  
**Problem**: Default function returns owned String
```rust
fn default_classes() -> String {
    "text-foreground w-6 h-6".to_string()  // 🚨 Heap allocation
}
```

**Solution**: Use constant
```rust
const DEFAULT_CLASSES: &str = "text-foreground w-6 h-6";

fn default_classes() -> &'static str {
    DEFAULT_CLASSES
}
```

### 4. SVG String Manipulation Optimization
**File**: `server/src/routes/icons.rs:42-60`  
**Problem**: Multiple string copies in SVG processing
```rust
fn add_classes_to_svg(svg_content: &str, classes: &str) -> String {
    let mut result = svg_content.to_string();  // 🚨 Full copy
    result.insert_str(class_end, &format!(" {}", classes));
    result
}
```

**Solution**: Use `Cow<str>` for conditional allocation
```rust
use std::borrow::Cow;

fn add_classes_to_svg(svg_content: &str, classes: &str) -> Cow<str> {
    if classes.is_empty() {
        return Cow::Borrowed(svg_content);  // No allocation needed
    }
    
    // Only allocate when modification is needed
    // ... rest of logic with Cow::Owned
}
```

### 5. Cookie Value String Allocation
**File**: `server/src/middleware.rs:35`  
**Problem**: Unnecessary string allocation for cookie value
```rust
.map(|cookie| cookie.value().to_string());
```

**Solution**: Work with &str if possible, or clone only when needed

### 6. Token Cookie Conversion
**File**: `server/src/routes/themes.rs:153`  
**Problem**: Unnecessary string conversion
```rust
let token_owned = token.to_string();
```

**Solution**: Cookie::build can work with &str directly

## 🔧 **Priority 3: Low Impact**

### 7. Request Method/URI Cloning in Middleware
**File**: `server/src/middleware.rs:12-13`  
**Problem**: Cloning request components for logging
```rust
let method = req.method().clone();  // 🚨 Small but unnecessary
let uri = req.uri().clone();        // 🚨 Small but unnecessary
```

**Solution**: Use references directly
```rust
println!("{} {} {} - {:?}", req.method(), req.uri(), status.as_u16(), duration);
```

### 8. Database Configuration String Allocations
**File**: `server/src/database.rs:10-13`  
**Problem**: Multiple environment variable string allocations
```rust
let db_host = env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
```

**Solution**: Consider using `Cow<str>` or lazy static for defaults
```rust
const DEFAULT_DB_HOST: &str = "127.0.0.1";
let db_host = env::var("DB_HOST").unwrap_or_else(|_| DEFAULT_DB_HOST.to_string());
```

### 9. JWT Secret Key Default
**File**: `server/src/token.rs:9`  
**Problem**: Default secret key string allocation
```rust
"dev_only_insecure_key_change_in_production".to_string()
```

**Solution**: Use const for default value

### 10. Database Version String Processing
**File**: `server/src/database.rs:40`  
**Problem**: Complex string processing for display
```rust
version.split_whitespace().take(2).collect::<Vec<_>>().join(" v").to_lowercase(),
```

**Solution**: Direct formatting without intermediate Vec allocation

## 📊 **Impact Summary**

**Estimated Performance Gains**:
- **High Priority**: 60-80% reduction in icon endpoint allocations
- **Medium Priority**: 20-40% reduction in theme-related allocations
- **Low Priority**: 5-15% reduction in various small allocations

**Implementation Difficulty**:
- **High Priority**: Easy - add `once_cell` dependency, define constants
- **Medium Priority**: Moderate - may require API signature changes
- **Low Priority**: Easy - simple refactoring



