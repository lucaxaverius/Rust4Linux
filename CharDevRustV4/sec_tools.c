#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>
#include <stdlib.h>
#include <sys/ioctl.h>
#include <stdint.h>

// Define the IOCTL commands
#define IOCTL_MAGIC 's'
#define IOCTL_ADD_RULE _IOW(IOCTL_MAGIC, 1, struct IoctlArgument)
#define IOCTL_REMOVE_RULE _IOW(IOCTL_MAGIC, 2, struct IoctlArgument)

#define DEVICE_PATH "/dev/secrules"
#define RULE_SIZE 256
#define BUFFER_SIZE RULE_SIZE*1000

typedef uint32_t u32 ;

// Updated struct to match the kernel module
struct IoctlArgument {
    u32 uid;             // User ID
    char rule[RULE_SIZE]; // Rule string
} typedef IoctlArgument;


int create_ioctl_argument(u32 uid, const char *rule, IoctlArgument *arg);
void ioctl_add_rule(u32 uid, const char *rule);
void ioctl_remove_rule(u32 uid, const char *rule);
void print_rules();

// Function to sanitize input and create IoctlArgument
int create_ioctl_argument(u32 uid, const char *rule, IoctlArgument *arg) {
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

void ioctl_add_rule(u32 uid, const char *rule) {
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

void ioctl_remove_rule(u32 uid, const char *rule) {
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

int main(int argc, char *argv[]) {
    if (argc < 2) {
        printf("Usage: %s <print|ioctl_add|ioctl_remove> [uid] [rule]\n", argv[0]);
        return -1;
    }

    if (strcmp(argv[1], "print") == 0) {
        print_rules();
    } else if (strcmp(argv[1], "ioctl_add") == 0) {
        if (argc != 4) {
            printf("Usage: %s ioctl_add <uid> <rule>\n", argv[0]);
            return -1;
        }
        u32 uid = (u32)atoi(argv[2]);
        ioctl_add_rule(uid, argv[3]);
    } else if (strcmp(argv[1], "ioctl_remove") == 0) {
        if (argc != 4) {
            printf("Usage: %s ioctl_remove <uid> <rule>\n", argv[0]);
            return -1;
        }
        u32 uid = (u32)atoi(argv[2]);
        ioctl_remove_rule(uid, argv[3]);
    } else {
        printf("Unknown command %s\n", argv[1]);
    }

    return 0;
}
