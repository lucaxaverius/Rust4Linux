#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>
#include <stdlib.h>
#include <sys/ioctl.h>
#include <stdint.h>

// Define the IOCTL commands
#define IOCTL_MAGIC 's'
#define IOCTL_ADD_RULE _IOW(IOCTL_MAGIC, 1, IoctlArgument)
#define IOCTL_REMOVE_RULE _IOW(IOCTL_MAGIC, 2, IoctlArgument)
#define IOCTL_READ_RULES _IOR(IOCTL_MAGIC, 3, IoctlReadArgument)

#define DEVICE_PATH "/dev/secrules"
#define RULE_SIZE 256
#define BUFFER_SIZE RULE_SIZE*16

typedef uint32_t u32 ;

// Updated struct to match the kernel module
struct IoctlArgument {
    u32 uid;             // User ID
    char rule[RULE_SIZE]; // Rule string
} typedef IoctlArgument;

struct IoctlReadArgument {
    u32 uid;         // User ID (MAX U32 indicates no specific user ID)
    char buffer[BUFFER_SIZE]; // Buffer to store rules
} typedef IoctlReadArgument;

int create_ioctl_argument(u32 uid, const char *rule, IoctlArgument *arg);
int create_ioctl_read_argument(u32 uid, IoctlReadArgument *arg);
void add_rule(u32 uid, const char *rule);
void remove_rule(u32 uid, const char *rule);
void print_man();
void print_rules();
void print_rules_by_id(u32 uid);
int get_command(const char* command);

// Function to sanitize input and create IoctlArgument
int create_ioctl_argument(u32 uid, const char *rule, IoctlArgument *arg) {
    
    // Validate that arg is not NULL
    if (!arg) {
        fprintf(stderr, "Error: IoctlArgument is NULL.\n");
        return -1;
    }

    // Validate that rule is not NULL
    if (!rule) {
        fprintf(stderr, "Error: Rule string is NULL.\n");
        return -1;
    }

    // Validate that the rule is a valid UTF-8 string and does not exceed RULE_SIZE
    size_t rule_len = strnlen(rule, RULE_SIZE - 1);  // -1 to leave space for the null terminator

    // Check if the rule is within the allowed length and does not contain interior NULs
    if (rule_len == RULE_SIZE - 1 && rule[rule_len] != '\0') {
        fprintf(stderr, "Error: Rule string is too long or contains interior NUL bytes.\n");
        return -1;
    }

    // Initialize the IoctlArgument structure
    memset(arg, 0, sizeof(IoctlArgument));
    arg->uid = uid;

    // Copy the rule to the IoctlArgument structure, copying only up to the actual length of the rule
    memcpy(arg->rule, rule, rule_len);

    // Ensure the last character is a NUL terminator
    arg->rule[rule_len] = '\0';

    return 0; // Success
}

// Function to sanitize input and create IoctlReadArgument
int create_ioctl_read_argument(u32 uid, IoctlReadArgument *arg) {

    // Validate that arg is not NULL
    if (!arg) {
        fprintf(stderr, "Error: IoctlReadArgument is NULL.\n");
        return -1;
    }
    
    // Initialize the IoctlArgument structure
    memset(arg, 0, sizeof(IoctlReadArgument));
    arg->uid = uid;

    // Allocate memory for the rules_buffer
    // Ensure this matches the size expected in the kernel (RULE_BUFFER_SIZE)
    memset(arg->buffer, 0, BUFFER_SIZE);

    return 0; // Success
}

// Function to map command strings to integer values
int get_command(const char *command) {
    if (strcmp(command, "print") == 0) return 1;
    if (strcmp(command, "add") == 0) return 2;
    if (strcmp(command, "rmv") == 0) return 3;
    if (strcmp(command, "man") == 0) return 4;
    return 0; // Unknown command
}

// Function to add new rule
void add_rule(u32 uid, const char *rule) {
    int fd = open(DEVICE_PATH, O_WRONLY);
    if (fd < 0) {
        perror("Failed to open the device");
        return;
    }

    struct IoctlArgument arg;
    int ret = create_ioctl_argument(uid, rule, &arg);
    if (ret < 0) {
        perror("Bad arguments");
        return; // Error already logged in create_ioctl_argument
    }

    if (ioctl(fd, IOCTL_ADD_RULE, &arg) < 0) {
        perror("Failed to add rule via ioctl");
    }

    close(fd);
}

// Function to remove a rule from a specific user
void remove_rule(u32 uid, const char *rule) {
    int fd = open(DEVICE_PATH, O_WRONLY);
    if (fd < 0) {
        perror("Failed to open the device");
        return;
    }

    struct IoctlArgument arg;
    int ret = create_ioctl_argument(uid, rule, &arg);
    if (ret < 0) {
        perror("Bad arguments");
        return; // Error already logged in create_ioctl_argument
    }

    if (ioctl(fd, IOCTL_REMOVE_RULE, &arg) < 0) {
        perror("Failed to remove rule via ioctl");
    }

    close(fd);
}

// Function to retrieve all the rules and the will print that
void print_rules() {
    int fd = open(DEVICE_PATH, O_RDONLY);
    if (fd < 0) {
        perror("Failed to open the device");
        return;
    }
    char buffer[BUFFER_SIZE];
    int bytes_read = read(fd, buffer, BUFFER_SIZE - 1);
    if (bytes_read < 0) {
        perror("Failed to read from the device");
        close(fd);
        return;
    }
    buffer[bytes_read] = '\0';
    printf("%s", buffer);
    close(fd);
}

void print_rules_by_id(u32 uid){
    int fd = open(DEVICE_PATH, O_WRONLY);
    if (fd < 0) {
        perror("Failed to open the device");
        return;
    }

    struct IoctlReadArgument arg;
    int ret = create_ioctl_read_argument(uid, &arg);
    if (ret < 0) {
        perror("Bad arguments");
        return; // Error already logged in create_ioctl_read_argument
    }

    if (ioctl(fd, IOCTL_READ_RULES, &arg) < 0) {
        perror("Failed to add rule via ioctl");
    }

    printf("%s", arg.buffer);

    close(fd);
}

// Helper function to print the manual
void print_man() {
    printf("Command Manual:\n");
    printf("1. print - Print all current rules. Works for all of a specific user.\n");
    printf("   Usage: sec_tool print\nsec_tool print uid ");
    printf("2. add - Add a rule for a specific user ID (uid).\n");
    printf("   Usage: sec_tool add <uid> <rule>\n");
    printf("3. rmv - Remove a rule for a specific user ID (uid).\n");
    printf("   Usage: sec_tool rmv <uid> <rule>\n");
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        printf("Usage: %s <print|add|rmv|man> [uid] [rule]\n", argv[0]);
        return -1;
    }

    switch (get_command(argv[1])) {
        case 1: // print
            if (argc == 4){
                // The second argument is not needed
                printf("Usage: %s print <uid> >\n", argv[0]);
                return -1;            
            }
            if(argc == 3){
                // Takes the UID and prints only the rules associated to it
                print_rules_by_id((u32)atoi(argv[2]));
            }
            else{
                // Print all the rules
                print_rules();
            }

            break;
        case 2: // add
            if (argc != 4) {
                printf("Usage: %s add <uid> <rule>\n", argv[0]);
                return -1;
            }
            add_rule((u32)atoi(argv[2]), argv[3]);
            break;
        case 3: // rmv
            if (argc != 4) {
                printf("Usage: %s rmv <uid> <rule>\n", argv[0]);
                return -1;
            }
            remove_rule((u32)atoi(argv[2]), argv[3]);
            break;
        case 4: // man
            print_man();
            break;
        default:
            printf("Unknown command %s\n", argv[1]);
            return -1;
    }

    return 0;
}