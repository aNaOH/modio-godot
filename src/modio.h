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

	bool init(String key, unsigned int game);

	void _process(double delta) override;
};

}

#endif