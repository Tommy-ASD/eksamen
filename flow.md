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

## Step 3: Post-login

After having logged in, the user gets 3 choices;

    1. Add a new wish to their account

    2. Read their own wishlist

    3. Read another user's wishlist

### Adding a new wish

Should the user choose to add a new wish, the program gets the ID of the user's account. This will be used later to add a graph connection from the user to the wishes they create.

After getting the user ID, the program asks them to provide an encryption key for this new wish.

It then creates the item, and adds a graph connection from the user's account to the item, calling the connection "wishes_for". The user can then choose to add more wishes.

### Read their own wishlist

The user is asked to provide a decryption key to their wishes. Once inputed, the program filters out all wishes that did not match that decryption key.

The remaining wishes are printed to the console.

### Read another user's wishlist

The user is first asked to provide the username of the person who's wishes they want to see. Afterwards, they are asked to provide the decryption key to the wishes.

As with the function that reads one's own wishes, the wishes that do not match that decryption key are filtered out.

The user is then asked to provide a max price, where 0 means no limit. If a max price is provided, the program filters out all wishes that cost more than that.

The remaining wishes are then sorted by price in an ascending order and printed, along with an index.

To end off, the user is asked if they want to mark any of the wishes as bought. If yes, they are asked which index to mark, and that wish is marked as bought.
