# Flowerchat protocol implementation

This crate defines flowerchat protocol and provides official rust primitives
for it.

# Flowerchat v1 protocol

## Base types definitions

```ts
type Hash = string; // libflowerpot v1 hash
type PublicKey = string; // libflowerpot v1 public key

type Role = 'owner' | 'administrator' | 'moderator' | 'user';

type Space = {
    // Information about the root block of the space's blockchain.
    root_block: {
        // Hash of the root block.
        hash: Hash;

        // Public key of the root block's author.
        public_key: PublicKey;
    };

    // Space metadata for clients UI.
    metadata: {
        // Title of the space.
        title: string;

        // Description of the space.
        description: string;

        // Space rules which new users will have to agree with before joining.
        rules: string;
    };

    // Space permissions for content moderation.
    permissions: {
        users: {
            // Who can change users' names besides the users themselves.
            rename: Role;

            // Who can ban users space-wide.
            ban: Role;
        };

        public_rooms: {
            // Who can create new public rooms.
            create: Role;

            // Who can rename existing public rooms.
            rename: Role;

            // Who can delete public rooms.
            delete: Role;
        };
    };
};

type PublicRoom = {
    // Unique name of the public room.
    name: string;

    // Blockchain information about the room.
    reference: {
        // Hash of the block where the room was created.
        block_hash: Hash;

        // Hash of transaction within this block.
        transaction_hash: Hash;

        // Public key of the room's creator.
        public_key: PublicKey;
    };

    // Room metadata for clients UI.
    metadata: {
        // Title of the room.
        title: string;

        // Description of the room.
        description: string;

        // Room rules which new users will have to agree with before joining.
        rules: string;
    };

    // Public room permissions for content moderation.
    permissions: {
        users: {
            // Who can ban the users of the public room.
            ban: Role;
        };

        messages: {
            // Who can delete users' messages.
            delete: Role;
        };
    };
};

type PublicRoomMessage = {
    // Name of the public room this message belongs to.
    room_name: string;

    // Text of the message.
    content: string;

    // Blockchain information about the message.
    reference: {
        // Hash of the block where the message was sent.
        block_hash: Hash;

        // Hash of transaction within this block.
        transaction_hash: Hash;

        // Public key of the message's sender.
        public_key: PublicKey;
    };
};
```

## Events

```ts
type CreatePublicRoom = {
    name: 'v1.rooms.user.create_public';
    body: {
        name: string;
    };
};

type PublicRoomMessage = {
    name: 'v1.rooms.user.public_message';
    body: {
        room_name: string;
        message: string;
    };
};
```

Author: [Nikita Podvirnyi](https://github.com/krypt0nn)\
Licensed under [GPL-3.0](LICENSE)
