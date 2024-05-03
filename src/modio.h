#ifndef GDMODIO_H
#define GDMODIO_H

#include <godot_cpp/classes/node.hpp>

namespace godot {

class ModIO : public Node {
	GDCLASS(ModIO, Node)

protected:
	static void _bind_methods();

public:
	ModIO();
	~ModIO();

	void init(String key, unsigned int game);

	void has_init(bool ok);

	void _process(double delta) override;
};

}

#endif