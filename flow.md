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

After having logged in, the user gets 4 choices;

    1. Add a new wish to their account

    2. Read their own wishlist

    3. Read another user's wishlist

    4. View their notifications

### Adding a new wish

Should the user choose to add a new wish, the program gets the ID of the user's account. This will be used later to add a graph connection from the user to the wishes they create.

After getting the user ID, the program asks them to provide an encryption key for this new wish.

It then creates the item, and adds a graph connection from the user's account to the item, calling the connection "wishes_for". The user can then choose to add more wishes.

### Read their own wishlist

The user is asked to provide a decryption key to their wishes. Once inputed, the program filters out all wishes that did not match that decryption key.

The remaining wishes are printed to the console, along with an index. The user is then asked if they want to delete any wishes.

#### Delete wish

If the user answers yes, they are asked to provide an index to delete. After the index is verified, the wish is deleted.

(NOTE: There is currently a bug with the database where all aspects of a connection are deleted once the connection is deleted. This means that the user's account is also deleted. This bug has been fixed, but the fix has not yet been pushed to the main branch. Source; https://github.com/surrealdb/surrealdb/issues/1726)

The user to item connection table has an event that is triggered every time an element is deleted, which creates a new "Notification" element containing the item along with the item's fields, the same for the user, and the recipients of the notification.

### Read another user's wishlist

The user is first asked to provide the username of the person who's wishes they want to see. Afterwards, they are asked to provide the decryption key to the wishes.

As with the function that reads one's own wishes, the wishes that do not match that decryption key are filtered out.

The user is then asked to provide a max price, where 0 means no limit. If a max price is provided, the program filters out all wishes that cost more than that.

The remaining wishes are then sorted by price in an ascending order and printed, along with an index.

If the item has been bought by someone, the user will be notified of it.

To end off, the user is asked if they want to mark any of the wishes as bought. If yes, they are asked which index to mark, and that wish is marked as bought.

### View notifications

All notifications are selected from the database, and the ones that don't have the current user as a recipient are filtered out.

The remaining notifications are all printed, and afterwards the user is removed as a recipient, as they have now seen the notification.
