## User CLI Guide: Security Rules Management

The `sec_tool` is a user-space command-line interface (CLI) utility for interacting with the security rules device. It allows users to add, remove, and retrieve access control rules associated with specific user IDs (UID) via IOCTL commands. <br /> 
**N.B.** Must be super user to interact with the device. 

### Usage

sec_tool <command> [uid] [rule]


### Commands

- **`print`**: Print all rules or the rules associated with a specific user ID.
  - **Usage**: 
    - `sec_tool print` - Prints all current rules.
    - `sec_tool print <uid>` - Prints only the rules associated with the specified `<uid>`.
  
- **`add`**: Add a new rule for a specific user ID.
  - **Usage**: 
    - `sec_tool add <uid> <rule>` - Adds the `<rule>` for the user identified by `<uid>`.

- **`rmv`**: Remove an existing rule for a specific user ID.
  - **Usage**: 
    - `sec_tool rmv <uid> <rule>` - Removes the `<rule>` associated with the user identified by `<uid>`.

- **`man`**: Display the command manual.
  - **Usage**: 
    - `sec_tool man` - Displays this manual.

### Examples

1. **Print all rules:**
   ```bash
   sec_tool print

2. **Add a new rule:**
    ```bash
   sec_tool add 1001 "Allow SSH Access"
