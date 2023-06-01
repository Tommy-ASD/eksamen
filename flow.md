# Wishlist

## Step 1: Setup

The program starts up the database executable as a child process, then connects to the database at localhost:8000

## Step 2: Login

The user is prompted with 2 options; signin or signup.

After this, the user is prompted to enter a username and a password.

### Signin

If user chooses signin, the program checks if the provided credentials are valid. If not, we return to the beginning of this step.

### Signup

If user chooses signup, a new account is created with the provided credentials.
